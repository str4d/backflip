use clap::Subcommand;
use color_eyre::eyre::Result;

mod analyze;

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum Command {
    /// Analyze a .ir file containing RAW samples from a single transmitter.
    Analyze(analyze::Command),
}

impl Command {
    pub(crate) fn run(self) -> Result<()> {
        match self {
            Command::Analyze(cmd) => cmd.run(),
        }
    }
}
