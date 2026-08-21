use clap::{Parser, Subcommand};
use prism::render_demo;
use std::fs;

#[derive(Parser)]
#[command(name = "prism", about = "A from-scratch software 3D rasterizer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render the built-in demo scene to a PPM file.
    Render {
        #[arg(short, long, default_value = "out.ppm")]
        output: String,
        #[arg(long, default_value_t = 640)]
        width: usize,
        #[arg(long, default_value_t = 480)]
        height: usize,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Render { output, width, height } => {
            let fb = render_demo(width, height);
            fs::write(&output, fb.to_ppm()).expect("failed to write output file");
            println!("wrote {} ({}x{})", output, width, height);
        }
    }
}
