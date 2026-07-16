use clap::{Parser, Subcommand};
use refetch_contract::RankRequest;
use std::{fs, path::PathBuf};
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Rank {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        Command::Rank { input, output } => {
            let data = fs::read_to_string(input)?;
            let req: RankRequest = serde_json::from_str(&data)?;
            let slate = refetch_core::rank(&req)?;
            fs::write(output, serde_json::to_string_pretty(&slate)? + "\n")?;
        }
    }
    Ok(())
}
