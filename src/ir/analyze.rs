use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::{eyre, Result};

use crate::ir::ButtonKind;

use super::parse;

#[derive(Clone, Debug, Args)]
pub(crate) struct Command {
    /// Path to the .ir file to analyze.
    file: PathBuf,
}

impl Command {
    pub(super) fn run(self) -> Result<()> {
        // TODO
        Ok(())
    }
}
