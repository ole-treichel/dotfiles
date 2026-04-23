use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod config;
mod surfaces;

use config::Config;
use surfaces::{Mode, SurfaceReport};

#[derive(Parser)]
#[command(
    name = "theme",
    about = "Switch system-wide light/dark mode across GNOME, Neovim, tmux, and GNOME Terminal.",
    version
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Switch everything to light mode.
    Light,
    /// Switch everything to dark mode.
    Dark,
    /// Flip the current mode.
    Toggle,
    /// Print the current system color-scheme.
    Status,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let cfg = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("config: {e:#}");
            return ExitCode::from(2);
        }
    };

    let target = match cli.cmd {
        Cmd::Light => Mode::Light,
        Cmd::Dark => Mode::Dark,
        Cmd::Toggle => match surfaces::system::read() {
            Ok(m) => m.flip(),
            Err(e) => {
                eprintln!("toggle: cannot read current mode: {e:#}");
                return ExitCode::from(2);
            }
        },
        Cmd::Status => {
            match surfaces::system::read() {
                Ok(m) => println!("{}", m.label()),
                Err(e) => {
                    eprintln!("status: {e:#}");
                    return ExitCode::from(2);
                }
            }
            return ExitCode::SUCCESS;
        }
    };

    let reports: Vec<SurfaceReport> = vec![
        surfaces::system::apply(target, &cfg),
        surfaces::nvim::apply(target, &cfg),
        surfaces::tmux::apply(target, &cfg),
        surfaces::gnome_terminal::apply(target, &cfg),
    ];

    let mut any_err = false;
    for r in &reports {
        match &r.outcome {
            Ok(msg) => println!("{:<16} ok  {msg}", r.name),
            Err(e) => {
                any_err = true;
                eprintln!("{:<16} ERR {e:#}", r.name);
            }
        }
    }

    if any_err {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
