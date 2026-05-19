//! Integration tests for root-vortex.
//!
//! Tests create a synthetic ROOT file using oxyroot, convert it to Vortex,
//! and verify the output file exists and has a non-zero size.

use std::fs;
use tempfile::NamedTempFile;
use tempfile::TempDir;

use oxyroot::{RootFile, WriterTree};
use root_vortex::converter::convert_root_to_vortex;
use root_vortex::info::tree_info;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a small ROOT file containing three branches:
///   `x` (i32), `y` (f32), `label` (String)
fn create_test_root_file(path: &str) -> anyhow::Result<()> {
    let mut file = RootFile::create(path)?;
    let mut tree = WriterTree::new("test_tree");

    let x_iter = vec![1_i32, 2, 3, 4, 5].into_iter();
    let y_iter = vec![1.1_f32, 2.2, 3.3, 4.4, 5.5].into_iter();
    let label_iter = vec![
        "alpha".to_string(),
        "beta".to_string(),
        "gamma".to_string(),
        "delta".to_string(),
        "epsilon".to_string(),
    ]
    .into_iter();

    tree.new_branch("x", x_iter);
    tree.new_branch("y", y_iter);
    tree.new_branch("label", label_iter);

    tree.write(&mut file)?;
    file.close()?;
    Ok(())
}

/// Write a ROOT file containing every supported primitive type.
fn create_primitives_root_file(path: &str) -> anyhow::Result<()> {
    let mut file = RootFile::create(path)?;
    let mut tree = WriterTree::new("prims");

    tree.new_branch("bi32", vec![10_i32, 20, 30].into_iter());
    tree.new_branch("bf32", vec![1.0_f32, 2.0, 3.0].into_iter());
    tree.new_branch("bf64", vec![1.1_f64, 2.2, 3.3].into_iter());
    tree.new_branch("bi64", vec![100_i64, 200, 300].into_iter());
    tree.new_branch("bu32", vec![1_u32, 2, 3].into_iter());

    tree.write(&mut file)?;
    file.close()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests: info
// ---------------------------------------------------------------------------

#[test]
fn test_tree_info_basic() -> anyhow::Result<()> {
    let tmp = NamedTempFile::with_suffix(".root")?;
    let root_path = tmp.path().to_str().unwrap().to_string();
    create_test_root_file(&root_path)?;

    let info = tree_info(&root_path, "test_tree")?;
    assert_eq!(info.name, "test_tree");
    assert_eq!(info.entries, 5);
    assert_eq!(info.branches.len(), 3);

    let names: Vec<_> = info.branches.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"x"), "expected branch 'x'");
    assert!(names.contains(&"y"), "expected branch 'y'");
    assert!(names.contains(&"label"), "expected branch 'label'");

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests: conversion
// ---------------------------------------------------------------------------

#[test]
fn test_convert_basic() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let root_path = dir.path().join("test.root");
    let vortex_path = dir.path().join("test.vortex");

    create_test_root_file(root_path.to_str().unwrap())?;

    let summary = convert_root_to_vortex(&root_path, &vortex_path, "test_tree")?;

    assert_eq!(summary.entries, 5, "expected 5 entries");
    // x, y, label → 3 columns
    assert_eq!(summary.columns_written, 3, "expected 3 columns written");
    assert!(summary.columns_skipped.is_empty(), "expected no skipped columns");
    assert!(
        vortex_path.exists(),
        "output vortex file should exist"
    );
    assert!(
        fs::metadata(&vortex_path)?.len() > 0,
        "output vortex file should be non-empty"
    );

    Ok(())
}

#[test]
fn test_convert_all_primitives() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let root_path = dir.path().join("prims.root");
    let vortex_path = dir.path().join("prims.vortex");

    create_primitives_root_file(root_path.to_str().unwrap())?;

    let summary = convert_root_to_vortex(&root_path, &vortex_path, "prims")?;

    assert_eq!(summary.entries, 3);
    assert_eq!(summary.columns_written, 5);
    assert!(summary.columns_skipped.is_empty());
    assert!(vortex_path.exists());

    Ok(())
}

#[test]
fn test_convert_missing_tree_fails() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let root_path = dir.path().join("test.root");
    let vortex_path = dir.path().join("test.vortex");

    create_test_root_file(root_path.to_str().unwrap())?;

    let result = convert_root_to_vortex(&root_path, &vortex_path, "no_such_tree");
    assert!(result.is_err(), "expected error for missing tree");

    Ok(())
}

#[test]
fn test_convert_missing_root_file_fails() {
    let result = convert_root_to_vortex("/tmp/does_not_exist.root", "/tmp/out.vortex", "tree");
    assert!(result.is_err(), "expected error for missing ROOT file");
}
