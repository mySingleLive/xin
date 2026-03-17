use clap::{Parser, Subcommand};
use std::path::PathBuf;
use xin::compiler::Compiler;

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
        /// Only generate object file
        #[arg(long)]
        emit_obj: bool,
    },
    /// Run a Xin source file directly (compile and execute)
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
        Commands::Compile {
            input,
            output,
            emit_ir,
            emit_obj,
        } => {
            let output_path = output.unwrap_or_else(|| {
                let input_name = input.file_stem().unwrap_or(std::ffi::OsStr::new("a.out"));
                PathBuf::from(input_name)
            });

            let compiler = Compiler::new()
                .with_emit_ir(emit_ir)
                .with_output(output_path);

            if emit_obj {
                // TODO: Implement emit_obj flag
                eprintln!("Warning: --emit-obj is not yet implemented");
            }

            compiler.compile(&input)?;
        }
        Commands::Run { input } => {
            // Compile to temp file and run
            let temp_dir = std::env::temp_dir();
            let pid = std::process::id();
            let output = temp_dir.join(format!("xin_run_{}", pid));

            let compiler = Compiler::new().with_output(output.clone());
            compiler.compile(&input)?;

            // Run the compiled executable
            let status = std::process::Command::new(&output)
                .status()
                .map_err(|e| anyhow::anyhow!("Failed to run executable: {}", e))?;

            // Cleanup
            let _ = std::fs::remove_file(&output);

            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Commands::Check { input } => {
            let compiler = Compiler::new().with_emit_ir(false);
            compiler.compile(&input)?;
            println!("Check passed!");
        }
    }

    Ok(())
}