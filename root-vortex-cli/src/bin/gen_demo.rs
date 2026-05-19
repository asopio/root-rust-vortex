//! Helper binary that generates a small demo ROOT file for CLI testing.
fn main() -> anyhow::Result<()> {
    use oxyroot::{RootFile, WriterTree};
    let path = std::env::args().nth(1).unwrap_or_else(|| "/tmp/demo.root".into());
    let mut file = RootFile::create(&path)?;
    let mut tree = WriterTree::new("events");
    tree.new_branch("run", vec![1_i32, 1, 2, 2, 3].into_iter());
    tree.new_branch("pt", vec![10.5_f32, 20.1, 30.7, 15.2, 22.8].into_iter());
    tree.new_branch("eta", vec![-1.5_f64, 0.3, 2.1, -0.7, 1.4].into_iter());
    tree.new_branch("channel", vec!["mu".to_string(), "e".to_string(), "mu".to_string(), "tau".to_string(), "e".to_string()].into_iter());
    tree.write(&mut file)?;
    file.close()?;
    eprintln!("Created {path}");
    Ok(())
}
