//! `root-vortex` CLI
//!
//! # Subcommands
//! - `convert`  – convert a ROOT TTree to a Vortex file
//! - `info`     – print schema/metadata for a ROOT TTree

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "root-vortex",
    about = "Convert CERN ROOT files to the Vortex columnar format",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging (repeat for more verbosity, e.g. -vv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a ROOT TTree to a Vortex file
    Convert {
        /// Path to the input ROOT file
        #[arg(short, long)]
        input: String,

        /// Path to the output Vortex file
        #[arg(short, long)]
        output: String,

        /// Name of the TTree inside the ROOT file
        #[arg(short, long, default_value = "tree")]
        tree: String,
    },

    /// Print branch schema and entry count for a ROOT TTree
    Info {
        /// Path to the ROOT file
        #[arg(short, long)]
        input: String,

        /// Name of the TTree inside the ROOT file
        #[arg(short, long, default_value = "tree")]
        tree: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialise logging based on -v flags
    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();

    match cli.command {
        Commands::Convert { input, output, tree } => {
            eprintln!("Converting '{}' (tree: '{}') → '{}'", input, tree, output);
            let summary = root_vortex::convert_root_to_vortex(&input, &output, &tree)?;
            eprintln!(
                "Done. {} entries, {} columns written{}",
                summary.entries,
                summary.columns_written,
                if summary.columns_skipped.is_empty() {
                    String::new()
                } else {
                    format!(", {} skipped: {}", summary.columns_skipped.len(),
                        summary.columns_skipped.join(", "))
                }
            );
        }

        Commands::Info { input, tree } => {
            let info = root_vortex::tree_info(&input, &tree)?;
            println!("Tree: {}  ({} entries)", info.name, info.entries);
            println!("{:<30} {:<25} Rust type", "Branch", "C++ type");
            println!("{}", "-".repeat(80));
            for b in &info.branches {
                println!(
                    "{:<30} {:<25} {}",
                    b.name, b.cpp_type, b.rust_type
                );
            }
        }
    }

    Ok(())
}
