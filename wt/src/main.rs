mod cmd;
mod config;
mod git;
mod hooks;
mod picker;
mod repo;
mod slug;
mod tms;

use clap::{Parser, Subcommand};

/// git worktree workflow for the bare-repo layout.
#[derive(Parser)]
#[command(name = "wt", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a branch + worktree, scaffold it, push, open a draft PR
    New {
        /// Words that become the branch and directory name
        #[arg(required = true, num_args = 1..)]
        words: Vec<String>,
        /// Base the branch on this ref instead of origin/HEAD
        #[arg(long, value_name = "REF")]
        from: Option<String>,
    },
    /// Check out an existing remote branch (picker if no branch given)
    Get {
        /// Remote branch name, verbatim
        branch: Option<String>,
    },
    /// Remove worktree(s) + local branch(es) (picker if no dir given)
    Rm {
        /// Worktree directories
        dirs: Vec<String>,
        /// Remove despite uncommitted changes or unpushed commits
        #[arg(short, long)]
        force: bool,
        /// Skip the confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// List worktrees
    Ls,
    /// Open the repo on GitHub (current branch, or the default branch at the
    /// workspace root)
    Open,
    /// Build the bare-repo layout from scratch
    Clone {
        /// Repository URL
        url: String,
        /// Directory name (defaults to the repository name)
        name: Option<String>,
    },
    /// Symlink docs/<slug>/ into the Obsidian vault (picker if none given)
    Vault {
        /// Worktree directories
        slugs: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::New { words, from } => cmd::new::run(&words, from),
        Command::Get { branch } => cmd::get::run(branch),
        Command::Rm { dirs, force, yes } => cmd::rm::run(&dirs, force, yes),
        Command::Ls => cmd::ls::run(),
        Command::Open => cmd::open::run(),
        Command::Clone { url, name } => cmd::clone::run(&url, name),
        Command::Vault { slugs } => cmd::vault::run(&slugs),
    };
    if let Err(e) = result {
        eprintln!("wt: {e:#}");
        std::process::exit(1);
    }
}
