//! Python bindings for `root-vortex`.
//!
//! Exposed as the `root_vortex` Python module.
//!
//! # Usage
//! ```python
//! import root_vortex
//!
//! # Convert a ROOT TTree to a Vortex file
//! summary = root_vortex.convert("events.root", "events.vortex", tree_name="events")
//! print(f"Wrote {summary.columns_written} columns, {summary.entries} rows")
//!
//! # Inspect a ROOT TTree without converting
//! info = root_vortex.tree_info("events.root", tree_name="events")
//! for branch in info.branches:
//!     print(branch.name, branch.cpp_type, branch.rust_type)
//! ```

// The #[pyfunction] macro generates implicit From<...> conversions that clippy
// misidentifies as "useless" for PyResult return types.
#![allow(clippy::useless_conversion)]

use pyo3::prelude::*;
use root_vortex_lib as rv;

// ---------------------------------------------------------------------------
// Python wrapper types
// ---------------------------------------------------------------------------

/// Summary of a completed ROOT → Vortex conversion.
#[pyclass(name = "ConversionSummary")]
#[derive(Clone)]
struct PyConversionSummary {
    inner: rv::converter::ConversionSummary,
}

#[pymethods]
impl PyConversionSummary {
    /// Number of rows (tree entries).
    #[getter]
    fn entries(&self) -> usize {
        self.inner.entries
    }

    /// Number of columns successfully written.
    #[getter]
    fn columns_written(&self) -> usize {
        self.inner.columns_written
    }

    /// Column names that were skipped (unsupported types).
    #[getter]
    fn columns_skipped(&self) -> Vec<String> {
        self.inner.columns_skipped.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "ConversionSummary(entries={}, columns_written={}, columns_skipped={:?})",
            self.inner.entries, self.inner.columns_written, self.inner.columns_skipped
        )
    }
}

/// Metadata for a single ROOT TTree branch.
#[pyclass(name = "BranchInfo")]
#[derive(Clone)]
struct PyBranchInfo {
    inner: rv::info::BranchInfo,
}

#[pymethods]
impl PyBranchInfo {
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    #[getter]
    fn cpp_type(&self) -> &str {
        &self.inner.cpp_type
    }

    #[getter]
    fn rust_type(&self) -> &str {
        &self.inner.rust_type
    }

    #[getter]
    fn entries(&self) -> i64 {
        self.inner.entries
    }

    fn __repr__(&self) -> String {
        format!(
            "BranchInfo(name={:?}, cpp_type={:?}, rust_type={:?}, entries={})",
            self.inner.name, self.inner.cpp_type, self.inner.rust_type, self.inner.entries
        )
    }
}

/// Metadata for a ROOT TTree.
#[pyclass(name = "TreeInfo")]
#[derive(Clone)]
struct PyTreeInfo {
    inner: rv::info::TreeInfo,
}

#[pymethods]
impl PyTreeInfo {
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    #[getter]
    fn entries(&self) -> i64 {
        self.inner.entries
    }

    #[getter]
    fn branches(&self) -> Vec<PyBranchInfo> {
        self.inner
            .branches
            .iter()
            .map(|b: &rv::info::BranchInfo| PyBranchInfo { inner: b.clone() })
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "TreeInfo(name={:?}, entries={}, branches={})",
            self.inner.name,
            self.inner.entries,
            self.inner.branches.len()
        )
    }
}

// ---------------------------------------------------------------------------
// Module functions
// ---------------------------------------------------------------------------

/// Convert a ROOT TTree to a Vortex file.
///
/// :param root_path: Path to the source ``.root`` file.
/// :param vortex_path: Path where the output ``.vortex`` file will be written.
/// :param tree_name: Name of the TTree inside the ROOT file (default ``"tree"``).
/// :returns: :class:`ConversionSummary`
/// :raises RuntimeError: if conversion fails.
#[pyfunction]
#[pyo3(signature = (root_path, vortex_path, tree_name="tree"))]
#[allow(clippy::useless_conversion)]
fn convert(
    root_path: &str,
    vortex_path: &str,
    tree_name: &str,
) -> PyResult<PyConversionSummary> {
    rv::convert_root_to_vortex(root_path, vortex_path, tree_name)
        .map(|inner| PyConversionSummary { inner })
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Inspect the schema of a ROOT TTree without converting.
///
/// :param root_path: Path to the ``.root`` file.
/// :param tree_name: Name of the TTree inside the ROOT file (default ``"tree"``).
/// :returns: :class:`TreeInfo`
/// :raises RuntimeError: if the file or tree cannot be opened.
#[pyfunction]
#[pyo3(signature = (root_path, tree_name="tree"))]
#[allow(clippy::useless_conversion)]
fn tree_info(root_path: &str, tree_name: &str) -> PyResult<PyTreeInfo> {
    rv::tree_info(root_path, tree_name)
        .map(|inner| PyTreeInfo { inner })
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

/// Python module for converting ROOT files to the Vortex columnar format.
#[pymodule]
fn root_vortex(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    m.add_function(wrap_pyfunction!(tree_info, m)?)?;
    m.add_class::<PyConversionSummary>()?;
    m.add_class::<PyBranchInfo>()?;
    m.add_class::<PyTreeInfo>()?;
    Ok(())
}
