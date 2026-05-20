//! Integration tests for root-vortex.

use std::fs;
use std::path::{Path, PathBuf};

use root_vortex::converter::convert_root_to_vortex;
use root_vortex::info::tree_info;
use tempfile::TempDir;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(name)
}

#[test]
fn test_tree_info_basic() -> anyhow::Result<()> {
    let info = tree_info(fixture_path("primitives.root"), "tree")?;
    assert_eq!(info.name, "tree");
    assert_eq!(info.entries, 3);
    assert_eq!(info.branches.len(), 3);

    let names: Vec<_> = info.branches.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"x"));
    assert!(names.contains(&"y"));
    assert!(names.contains(&"z"));

    Ok(())
}

#[test]
fn test_tree_info_vectors() -> anyhow::Result<()> {
    let info = tree_info(fixture_path("vectors.root"), "tree")?;
    assert_eq!(info.entries, 3);
    assert_eq!(info.branches.len(), 2);

    let vector_i32 = info
        .branches
        .iter()
        .find(|branch| branch.name == "vector_i32")
        .expect("vector_i32 branch");
    assert_eq!(vector_i32.rust_type, "Vec<i32>");

    let vector_f32 = info
        .branches
        .iter()
        .find(|branch| branch.name == "vector_f32")
        .expect("vector_f32 branch");
    assert_eq!(vector_f32.rust_type, "Vec<f32>");

    Ok(())
}

#[test]
fn test_convert_primitives() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let vortex_path = dir.path().join("primitives.vortex");

    let summary = convert_root_to_vortex(fixture_path("primitives.root"), &vortex_path, "tree")?;

    assert_eq!(summary.entries, 3);
    assert_eq!(summary.columns_written, 3);
    assert!(summary.columns_skipped.is_empty());
    assert!(vortex_path.exists());
    assert!(fs::metadata(&vortex_path)?.len() > 0);

    Ok(())
}

#[test]
fn test_convert_vectors() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let vortex_path = dir.path().join("vectors.vortex");

    let summary = convert_root_to_vortex(fixture_path("vectors.root"), &vortex_path, "tree")?;

    assert_eq!(summary.entries, 3);
    assert_eq!(summary.columns_written, 2);
    assert!(summary.columns_skipped.is_empty());
    assert!(vortex_path.exists());
    assert!(fs::metadata(&vortex_path)?.len() > 0);

    Ok(())
}

#[test]
fn test_convert_missing_tree_fails() {
    let dir = TempDir::new().expect("temp dir");
    let vortex_path = dir.path().join("missing-tree.vortex");

    let result = convert_root_to_vortex(fixture_path("primitives.root"), &vortex_path, "no_such_tree");
    assert!(result.is_err(), "expected error for missing tree");
}

#[test]
fn test_convert_missing_root_file_fails() {
    let dir = TempDir::new().expect("temp dir");
    let root_path = dir.path().join("does_not_exist.root");
    let vortex_path = dir.path().join("out.vortex");
    let result = convert_root_to_vortex(&root_path, &vortex_path, "tree");
    assert!(result.is_err(), "expected error for missing ROOT file");
}
