mod commands;
mod form;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = commands::Args::parse();

    match args.action {
        commands::Action::Preview { file } => {
            let form = form::deserialize(&file)?;
            println!("{:#?}", form);
        }
    }

    Ok(())
}
