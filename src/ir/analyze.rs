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
        let mut contents = String::new();
        File::open(self.file)?.read_to_string(&mut contents)?;
        let (_, buttons) =
            parse::ir_file(&contents).map_err(|e| eyre!("Invalid .ir file: {}", e))?;

        if buttons.is_empty() {
            return Err(eyre!("No buttons in .ir file"));
        }

        let raw = || {
            buttons.iter().filter_map(|b| match &b.kind {
                ButtonKind::Raw(raw) => Some(raw),
                _ => None,
            })
        };
        if raw().filter(|b| !b.data.is_empty()).count() == 0 {
            return Err(eyre!("No raw data in any of the buttons in .ir file"));
        }

        Ok(())
    }
}
