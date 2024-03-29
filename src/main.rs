use std::process::exit;

use clap::Parser;

mod cli;
mod error;
mod prompt;
mod req;
mod schema;
mod serde;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let res = cli.run();

    if let Err(e) = res {
        let (msg, code) = e.show();
        eprintln!("{}", msg);
        exit(code)
    }

    Ok(())
}
