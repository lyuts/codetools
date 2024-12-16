mod callgraph;
mod dataflow;

use anyhow::{self, Context};
use clap::{Args, Parser as ClapParser, Subcommand};
use std::io::Read;

#[derive(ClapParser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Callgraph(CallgraphArgs),
    DataFlow(DataFlowArgs),
}

#[derive(Args)]
struct CallgraphArgs {
    filepath: Option<String>,
}

#[derive(Args)]
struct DataFlowArgs {
    #[arg(long)]
    filepath: Option<String>,
    #[arg(long)]
    function: String,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match &args.command {
        Commands::Callgraph(cmd) => {
            let mut r: Box<dyn Read>;
            if let Some(path) = &cmd.filepath {
                r = Box::new(std::fs::File::open(path.clone())?);
            } else {
                r = Box::new(std::io::stdin());
            }
            let mut code = String::new();
            r.read_to_string(&mut code)?;

            let call_map = callgraph::find_function_calls(code.to_string())
                .context("Failed to process file.")?;

            for (k, v) in call_map.iter() {
                for f in v {
                    println!("{} calls {}", k, f);
                }
            }
        }
        Commands::DataFlow(cmd) => {
            let mut r: Box<dyn Read>;
            if let Some(path) = &cmd.filepath {
                r = Box::new(std::fs::File::open(path.clone())?);
            } else {
                r = Box::new(std::io::stdin());
            }
            let mut code = String::new();
            r.read_to_string(&mut code)?;

            let data = dataflow::find_accessible_data(&cmd.function, &code.to_string())
                .context("Failed to process file.")?;
            println!("{} has access to {:?}", cmd.function, data);
        }
    }
    Ok(())
}
