use clap::Parser;
use std::path::PathBuf;
use std::fs;
use rtex::{StreamingConverter, ConsoleReporter, watch_single, DocumentTemplate};

#[derive(Parser)]
#[command(name = "latex-rs")]
#[command(about = "Convert TeX files to PDF", long_about = None)]
struct Cli {
    #[arg(help = "Input TeX file path")]
    input: PathBuf,

    #[arg(short, long, help = "Output PDF file path")]
    output: Option<PathBuf>,

    #[arg(short, long, help = "Watch input file and recompile on changes")]
    watch: bool,

    #[arg(short, long, help = "Path to a TOML template file for styling")]
    template: Option<PathBuf>,
}

fn build_converter(cli: &Cli) -> anyhow::Result<StreamingConverter<ConsoleReporter>> {
    let mut converter = StreamingConverter::with_reporter(ConsoleReporter);
    if let Some(path) = &cli.template {
        let template = DocumentTemplate::from_toml(path)?;
        converter = converter.with_template(template);
    }
    Ok(converter)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let output = cli.output.clone().unwrap_or_else(|| {
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

    let input_path = cli.input.clone();
    if cli.watch {
        let template_path = cli.template.clone();
        let result: Result<(), anyhow::Error> = watch_single(&input_path, &output, move |inp, out| {
            let mut converter = StreamingConverter::with_reporter(ConsoleReporter);
            if let Some(path) = &template_path {
                let template = DocumentTemplate::from_toml(path)?;
                converter = converter.with_template(template);
            }
            converter.convert(inp, out)?;
            println!("Successfully converted {} to {}",
                     inp.display(),
                     out.display());
            Ok(())
        });
        // watch_single only returns on error; print it
        if let Err(e) = result {
            eprintln!("Watch mode error: {}", e);
            std::process::exit(1);
        }
    } else {
        let mut converter = build_converter(&cli)?;
        converter.convert(&input_path, &output)?;

        println!("Successfully converted {} to {}",
                 input_path.display(),
                 output.display());
    }

    Ok(())
}
