use clap::Subcommand;
use color_eyre::eyre::Result;

mod analyze;
mod parse;

#[derive(Debug)]
struct Button {
    name: String,
    kind: ButtonKind,
}

#[derive(Debug)]
enum ButtonKind {
    Parsed(ParsedButton),
    Raw(RawButton),
}

#[derive(Debug)]
struct ParsedButton {
    protocol: String,
    address: String,
    command: String,
}

#[derive(Debug)]
struct RawButton {
    frequency: u32,
    duty_cycle: f32,
    data: Vec<(u32, u32)>,
    final_on: Option<u32>,
}

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
