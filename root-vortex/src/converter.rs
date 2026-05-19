//! Core ROOT → Vortex conversion logic.
//!
//! The entry point is [`convert_root_to_vortex`].

use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result, bail};
use log::{debug, warn};
use oxyroot::RootFile;
use vortex::VortexSessionDefault;
use vortex::array::arrays::{PrimitiveArray, StructArray, VarBinViewArray};
use vortex::array::dtype::NativePType;
use vortex::array::{ArrayRef, IntoArray};
use vortex::file::WriteOptionsSessionExt;
use vortex::io::runtime::BlockingRuntime;
use vortex::io::runtime::current::CurrentThreadRuntime;
use vortex::io::session::RuntimeSessionExt;
use vortex::session::VortexSession;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Convert a ROOT TTree to a Vortex file.
///
/// # Arguments
/// * `root_path`   – path to the source `.root` file
/// * `vortex_path` – path where the output `.vortex` file will be written
/// * `tree_name`   – name of the TTree inside the ROOT file
///
/// # Errors
/// Returns an error if the ROOT file cannot be opened, the tree is not found,
/// any *supported* branch fails conversion, or writing the Vortex file fails.
/// Branches with unsupported types are skipped and reported in
/// [`ConversionSummary::columns_skipped`].
///
/// # Example
/// ```no_run
/// root_vortex::convert_root_to_vortex("events.root", "events.vortex", "events").unwrap();
/// ```
pub fn convert_root_to_vortex(
    root_path: impl AsRef<Path>,
    vortex_path: impl AsRef<Path>,
    tree_name: &str,
) -> Result<ConversionSummary> {
    let root_path = root_path.as_ref();
    let vortex_path = vortex_path.as_ref();

    debug!(
        "Opening ROOT file: {}",
        root_path.display()
    );
    let mut root_file =
        RootFile::open(root_path).with_context(|| format!("opening {}", root_path.display()))?;

    let tree = root_file
        .get_tree(tree_name)
        .with_context(|| format!("getting tree '{}' from {}", tree_name, root_path.display()))?;

    let raw_entries = tree.entries();
    let entries = usize::try_from(raw_entries)
        .with_context(|| format!("tree '{}' has invalid entry count: {}", tree_name, raw_entries))?;
    debug!("Tree '{}': {} entries", tree_name, entries);

    // Collect all top-level branches → (name, vortex array)
    let mut names: Vec<String> = Vec::new();
    let mut arrays: Vec<ArrayRef> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for branch in tree.branches() {
        let bname = branch.name().to_string();
        let interp = branch.interpretation();

        if !is_supported_interpretation(&interp) {
            warn!("Skipping branch '{}': unsupported branch type: {}", bname, interp);
            skipped.push(bname);
            continue;
        }

        let arr = branch_to_array(branch)
            .with_context(|| format!("converting branch '{}' ({})", bname, interp))?;
        debug!(
            "Branch '{}': {} → {} elements",
            bname,
            branch.item_type_name(),
            arr.len()
        );
        names.push(bname);
        arrays.push(arr);
    }

    if names.is_empty() {
        bail!("No convertible branches found in tree '{}'", tree_name);
    }

    // Build StructArray
    let struct_array = {
        let fields: Vec<(&str, ArrayRef)> = names
            .iter()
            .map(|n| n.as_str())
            .zip(arrays.iter().cloned())
            .collect();
        StructArray::from_fields(fields.as_slice())
            .context("building StructArray")?
    };

    debug!("Writing Vortex file: {}", vortex_path.display());

    // Create a blocking single-threaded Vortex runtime and write the file.
    // We must configure the session with a handle so that internal vortex
    // machinery (task scheduling, etc.) knows which executor to use.
    let runtime = CurrentThreadRuntime::new();
    let session = VortexSession::default().with_handle(runtime.handle());
    let out =
        File::create(vortex_path).with_context(|| format!("creating {}", vortex_path.display()))?;

    let array_ref = struct_array.into_array();
    let iter = array_ref.to_array_iterator();

    session
        .write_options()
        .blocking(&runtime)
        .write(out, iter)
        .context("writing Vortex file")?;

    Ok(ConversionSummary {
        entries,
        columns_written: names.len(),
        columns_skipped: skipped,
    })
}

/// Summary returned by a successful conversion.
#[derive(Debug, Clone)]
pub struct ConversionSummary {
    /// Number of tree entries (rows).
    pub entries: usize,
    /// Number of columns successfully written.
    pub columns_written: usize,
    /// Column names that were skipped (unsupported types).
    pub columns_skipped: Vec<String>,
}

// ---------------------------------------------------------------------------
// Branch → Array dispatch
// ---------------------------------------------------------------------------

/// Convert a single oxyroot `Branch` to a Vortex `ArrayRef`.
///
/// Dispatches on the Rust interpretation string returned by `branch.interpretation()`.
pub fn branch_to_array(branch: &oxyroot::Branch) -> Result<ArrayRef> {
    let interp = branch.interpretation();
    match interp.as_str() {
        "bool" => {
            // ROOT Bool_t — stored as 0/1; map to u8 (NativePType)
            let arr: PrimitiveArray = branch
                .as_iter::<bool>()?
                .map(|b| b as u8)
                .collect();
            Ok(arr.into_array())
        }
        "i8" => primitive_branch::<i8>(branch),
        "i16" => primitive_branch::<i16>(branch),
        "i32" => primitive_branch::<i32>(branch),
        "i64" => primitive_branch::<i64>(branch),
        "u8" => primitive_branch::<u8>(branch),
        "u16" => primitive_branch::<u16>(branch),
        "u32" => primitive_branch::<u32>(branch),
        "u64" => primitive_branch::<u64>(branch),
        "f32" => primitive_branch::<f32>(branch),
        "f64" => primitive_branch::<f64>(branch),
        "String" => {
            let vals: Vec<String> = branch.as_iter::<String>()?.collect();
            Ok(VarBinViewArray::from_iter_str(vals).into_array())
        }
        other => bail!("unsupported branch type: {}", other),
    }
}

fn is_supported_interpretation(interp: &str) -> bool {
    matches!(
        interp,
        "bool" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64"
            | "String"
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collect a primitive-typed branch into a `PrimitiveArray`.
///
/// Uses the `FromIterator<T>` impl on `PrimitiveArray`, which is available for all `T: NativePType`.
fn primitive_branch<T>(branch: &oxyroot::Branch) -> Result<ArrayRef>
where
    T: oxyroot::UnmarshalerInto<Item = T> + NativePType + 'static,
{
    let arr: PrimitiveArray = branch.as_iter::<T>()?.collect();
    Ok(arr.into_array())
}
