// Modified by ACRYL, 2026
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::cli::GlobalFlags;

#[derive(Args, Debug)]
pub struct ImportArgs {
    /// Import a .acrylmicrovm archive into a new box.
    #[arg(value_name = "ARCHIVE")]
    pub archive: PathBuf,
    #[arg(long)]
    pub name: Option<String>,
}

pub async fn execute(args: ImportArgs, global: &GlobalFlags) -> Result<()> {
    let runtime = global.create_runtime()?;
    let target = runtime
        .import_box(acrylmicrovm::BoxArchive::new(args.archive), args.name)
        .await?;
    println!("{}", target.id());
    Ok(())
}
