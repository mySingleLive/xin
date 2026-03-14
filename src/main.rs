use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xin")]
#[command(about = "Xin Programming Language Compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Xin source file to an executable
    Compile {
        /// Input source file
        input: PathBuf,
        /// Output executable path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Print intermediate representation
        #[arg(long)]
        emit_ir: bool,
    },
    /// Run a Xin source file directly
    Run {
        /// Input source file
        input: PathBuf,
    },
    /// Check syntax and types without generating code
    Check {
        /// Input source file
        input: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output, emit_ir } => {
            println!("Compiling: {:?}", input);
            if let Some(out) = output {
                println!("Output: {:?}", out);
            }
            if emit_ir {
                println!("Will emit IR");
            }
            // TODO: 实现编译逻辑
            println!("Compilation not yet implemented");
        }
        Commands::Run { input } => {
            println!("Running: {:?}", input);
            // TODO: 实现运行逻辑
            println!("Run not yet implemented");
        }
        Commands::Check { input } => {
            println!("Checking: {:?}", input);
            // TODO: 实现检查逻辑
            println!("Check not yet implemented");
        }
    }

    Ok(())
}