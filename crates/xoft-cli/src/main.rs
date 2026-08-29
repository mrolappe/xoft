use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use xoft_cli::corpus_run::{self, Allowlist};
use xoft_cli::manifest::{self, RootsConfig};
use xoft_cli::{check, transpile};

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
    /// Parse a source file and report its diagnostics
    Check { file: PathBuf },
    /// Parse a source file, report diagnostics, and round-trip it through the serializer
    Transpile {
        file: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
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
    /// Parse + round-trip every corpus file, honoring corpus/allowlist.toml (D8)
    Run {
        #[arg(long, default_value = "corpus/roots.toml")]
        roots: PathBuf,
        #[arg(long, default_value = "corpus/allowlist.toml")]
        allowlist: PathBuf,
        #[arg(long, default_value = "reports/corpus-report.json")]
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
            let m = manifest::build(&config.root);
            std::fs::write(&out, serde_json::to_string_pretty(&m)? + "\n")?;
            for r in &m.roots {
                println!("{:>18}  {:>4} files  {:>7} KB", r.alias, r.files, r.bytes / 1024);
            }
            println!("{:>18}  {:>4} files -> {}", "total", m.files.len(), out.display());
            for f in &m.failures {
                eprintln!("{:>18}  failed: {}", f.alias, f.error);
            }
            Ok(())
        }
        Command::Corpus {
            command: CorpusCommand::Run { roots, allowlist, out },
        } => {
            let config: RootsConfig = toml::from_str(
                &std::fs::read_to_string(&roots)
                    .with_context(|| format!("reading {}", roots.display()))?,
            )?;
            let allowlist: Allowlist = toml::from_str(
                &std::fs::read_to_string(&allowlist)
                    .with_context(|| format!("reading {}", allowlist.display()))?,
            )?;
            let report = corpus_run::run(&config.root, &allowlist)?;
            std::fs::write(&out, serde_json::to_string_pretty(&report)? + "\n")?;
            println!(
                "parse: {:.2}%  round-trip: {:.2}%  ({} counted, {} allowlisted, {} total) -> {}",
                report.parse_pct,
                report.round_trip_pct,
                report.counted_files,
                report.allowlisted_files,
                report.total_files,
                out.display()
            );
            if report.parse_ok == report.counted_files && report.round_trip_ok == report.counted_files {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        Command::Check { file } => {
            let result = check::check_file(&file)?;
            print!("{}", result.rendered);
            if result.diagnostics.is_empty() {
                println!("{}: OK", file.display());
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        Command::Transpile { file, out } => {
            let result = transpile::transpile_file(&file)?;
            print!("{}", result.check.rendered);
            match out {
                Some(out) => std::fs::write(&out, &result.output_bytes)
                    .with_context(|| format!("writing {}", out.display()))?,
                None => {
                    use std::io::Write;
                    std::io::stdout().write_all(&result.output_bytes)?;
                }
            }
            if result.check.diagnostics.is_empty() {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
    }
}
