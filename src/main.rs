use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

mod ir;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    category: Category,
}

#[derive(Clone, Debug, Subcommand)]
enum Category {
    /// Helpers for the 'infrared' app.
    #[command(subcommand)]
    Ir(ir::Command),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.category {
        Category::Ir(command) => command.run(),
    }
}
