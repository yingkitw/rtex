use clap::Parser;
use std::path::PathBuf;
use std::fs;
use latex_rs::convert_tex_to_pdf;

#[derive(Parser)]
#[command(name = "latex-rs")]
#[command(about = "Convert TeX files to PDF", long_about = None)]
struct Cli {
    #[arg(help = "Input TeX file path")]
    input: PathBuf,

    #[arg(short, long, help = "Output PDF file path")]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let output = cli.output.unwrap_or_else(|| {
        let output_dir = PathBuf::from("output");
        fs::create_dir_all(&output_dir).ok();
        
        let filename = cli.input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        output_dir.join(format!("{}.pdf", filename))
    });

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    convert_tex_to_pdf(&cli.input, &output)?;

    println!("Successfully converted {} to {}", 
             cli.input.display(), 
             output.display());

    Ok(())
}
