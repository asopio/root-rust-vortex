//! Utilities for inspecting the structure of a ROOT file without conversion.

use crate::RootFile;
use anyhow::{Context, Result};
use std::path::Path;

/// Metadata for a single TTree branch.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Branch name.
    pub name: String,
    /// C++ type name as reported by ROOT.
    pub cpp_type: String,
    /// Rust type interpretation.
    pub rust_type: String,
    /// Number of entries.
    pub entries: i64,
}

/// High-level description of a ROOT TTree.
#[derive(Debug, Clone)]
pub struct TreeInfo {
    /// Tree name.
    pub name: String,
    /// Total entries.
    pub entries: i64,
    /// All top-level branches.
    pub branches: Vec<BranchInfo>,
}

/// Open a ROOT file and return metadata for the named TTree.
///
/// # Example
/// ```no_run
/// let info = root_vortex::tree_info("events.root", "events").unwrap();
/// println!("{} entries, {} branches", info.entries, info.branches.len());
/// ```
pub fn tree_info(root_path: impl AsRef<Path>, tree_name: &str) -> Result<TreeInfo> {
    let root_path = root_path.as_ref();
    let mut root_file =
        RootFile::open(root_path).with_context(|| format!("opening {}", root_path.display()))?;
    let tree = root_file
        .get_tree(tree_name)
        .with_context(|| format!("getting tree '{}'", tree_name))?;

    let branches = tree
        .branches()
        .map(|b| BranchInfo {
            name: b.name().to_string(),
            cpp_type: b.item_type_name(),
            rust_type: b.interpretation(),
            entries: b.entries(),
        })
        .collect();

    Ok(TreeInfo {
        name: tree_name.to_string(),
        entries: tree.entries(),
        branches,
    })
}
