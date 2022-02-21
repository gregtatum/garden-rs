//! This file contains an experimental game client. It needs to be hooked up to
//! everything still.

use anyhow::{bail, Context, Result};
use std::{fs, path::PathBuf};

use garden::{
    chain_store::{FsChainStore, HeadRef},
    utils::path_join,
    Action,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "garden-cat",
    about = "List the contents of a chain, similar to the Unix cat command."
)]
struct CliOptions {
    /// The directory the garden files are persisted to.
    #[structopt()]
    head_ref_str: String,
}

fn main() -> Result<()> {
    let cli_options = CliOptions::from_args();
    let head_ref = HeadRef::try_from(cli_options.head_ref_str)
        .context("An invalid head ref was provided.")?;
    let path = PathBuf::from("./.garden");
    if !path.exists() {
        bail!("No .garden folder exists for this working directory");
    }

    let mut chain_store =
        FsChainStore::<Action>::try_new(path.clone(), head_ref.clone())?;

    chain_store.load_all_chains()?;

    let head_ref_path = path_join(chain_store.heads_path.clone(), &[head_ref.str()]);

    if !head_ref_path.exists() {
        let paths = fs::read_dir(chain_store.heads_path)
            .context("No .garden/heads path existed for the working directory.")?;

        let mut reason: String =
            format!("The reference {:?} does not exist.\n\n", head_ref.str());
        let mut is_first = true;
        for path in paths {
            if is_first {
                is_first = false;

                reason.push_str("The available head references are:\n");
            }
            let entry = path.context("Could not read path.")?;
            reason.push_str("  ");
            reason.push_str(&entry.file_name().to_string_lossy());
            reason.push_str("\n");
        }
        if is_first {
            bail!("No head refs have been created yet.");
        }

        bail!("{}", reason);
    }

    println!(
        "{}",
        serde_json::to_string_pretty(chain_store.chain.as_block_slice())?
    );

    Ok(())
}
