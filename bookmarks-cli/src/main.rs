#![feature(never_type)]
#![feature(if_let_guard)]
use std::{fs::File, io, path::PathBuf};

use clap::{Parser, Subcommand, command};
use mimalloc::MiMalloc;

mod tui;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Debug, Subcommand)]
enum Command {
    /// Interactive interface
    #[command(visible_alias = "fi")]
    Tui,

    /// Prints (L) best matched items [default]
    #[command(visible_alias = "f")]
    Find {
        #[arg(short = 'L', long, default_value = "3")]
        /// Limits output [0: all]
        limit: usize,

        /// Needle
        search: String,
    },

    /// Prints all stored bookmarks
    #[command(visible_alias = "p")]
    Print,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, global = true, default_value = "./sample-data")]
    data: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    if matches!(args.command, Some(Command::Tui) | None) {
        tracing_subscriber::fmt()
            .with_max_level(args.verbosity)
            .with_writer(File::create("./log.jsonl")?)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_writer(io::stderr)
            .with_max_level(args.verbosity)
            .init();
    }
    tracing::debug!("{args:?}");

    match args.command {
        Some(Command::Find { search, limit }) => {
            let data = bookmarks_data::load_from_fs(args.data)?
                .map(|it| it.content)
                .collect::<Vec<_>>();
            let res = bookmarks_data::search(&search, data.iter());
            let res = match limit {
                0 => res.take(usize::MAX),
                1.. => res.take(limit),
            };
            for (i, it) in res {
                let content = &data[i];
                println!("--- (score {it})");
                println!("{content:?}");
                println!("{}", content.fuzzy_string());
            }
        }
        Some(Command::Print) => {
            for file in bookmarks_data::load_from_fs(args.data)? {
                println!(
                    "{}: {}",
                    file.path.to_string_lossy(),
                    file.content.fuzzy_string()
                );
            }
        }
        Some(Command::Tui) | None => {
            tui::run(args.data)?;
        }
    }
    Ok(())
}
