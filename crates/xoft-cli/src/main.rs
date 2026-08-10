use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use xoft_cli::manifest::{self, RootsConfig};

#[derive(Parser)]
#[command(name = "xoft", about = "Oberon dialect workbench")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Corpus inventory and reporting
    Corpus {
        #[command(subcommand)]
        command: CorpusCommand,
    },
}

#[derive(Subcommand)]
enum CorpusCommand {
    /// Rebuild corpus/manifest.json from corpus/roots.toml
    Manifest {
        #[arg(long, default_value = "corpus/roots.toml")]
        roots: PathBuf,
        #[arg(long, default_value = "corpus/manifest.json")]
        out: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Corpus {
            command: CorpusCommand::Manifest { roots, out },
        } => {
            let config: RootsConfig = toml::from_str(
                &std::fs::read_to_string(&roots)
                    .with_context(|| format!("reading {}", roots.display()))?,
            )?;
            let m = manifest::build(&config.root)?;
            std::fs::write(&out, serde_json::to_string_pretty(&m)? + "\n")?;
            for r in &m.roots {
                println!("{:>18}  {:>4} files  {:>7} KB", r.alias, r.files, r.bytes / 1024);
            }
            println!("{:>18}  {:>4} files -> {}", "total", m.files.len(), out.display());
            Ok(())
        }
    }
}
