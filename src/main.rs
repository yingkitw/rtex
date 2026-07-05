use clap::Parser;
use std::path::PathBuf;
use std::fs;
use rtex::{
    ConversionOptions, OutputFormat, StreamingConverter, ConsoleReporter,
    watch_single, DocumentTemplate, convert_tex_file,
};

#[derive(Parser)]
#[command(name = "rtex")]
#[command(about = "Convert TeX files to PDF, HTML, DOCX, or EPUB", long_about = None)]
struct Cli {
    #[arg(help = "Input TeX file path")]
    input: PathBuf,

    #[arg(short, long, help = "Output file path")]
    output: Option<PathBuf>,

    #[arg(short, long, value_name = "FORMAT", default_value = "pdf", help = "Output format: pdf, html, docx, epub")]
    format: String,

    #[arg(long, help = "Fetch missing LaTeX packages from CTAN before conversion")]
    fetch_packages: bool,

    #[arg(long, help = "Directory for downloaded LaTeX package cache")]
    package_cache: Option<PathBuf>,

    #[arg(short, long, help = "Watch input file and recompile on changes")]
    watch: bool,

    #[arg(short, long, help = "Path to a TOML template file for styling")]
    template: Option<PathBuf>,

    #[arg(long, help = "Force rebuild even when source and dependencies are unchanged")]
    force: bool,

    #[arg(long, help = "Disable incremental compilation and always rebuild")]
    no_incremental: bool,
}

fn parse_format(raw: &str) -> anyhow::Result<OutputFormat> {
    raw.parse().map_err(|e: rtex::LatexError| anyhow::anyhow!(e.to_string()))
}

fn default_output(input: &PathBuf, format: OutputFormat) -> PathBuf {
    let output_dir = PathBuf::from("output");
    fs::create_dir_all(&output_dir).ok();
    let filename = input
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    output_dir.join(format!("{}.{}", filename, format.extension()))
}

fn conversion_options(cli: &Cli, format: OutputFormat) -> ConversionOptions {
    ConversionOptions {
        format,
        fetch_packages: cli.fetch_packages,
        package_cache: cli.package_cache.clone(),
    }
}

fn build_converter(cli: &Cli) -> anyhow::Result<StreamingConverter<ConsoleReporter>> {
    let mut converter = StreamingConverter::with_reporter(ConsoleReporter)
        .with_incremental(!cli.no_incremental)
        .with_force_rebuild(cli.force);
    if let Some(path) = &cli.template {
        let template = DocumentTemplate::from_toml(path)?;
        converter = converter.with_template(template);
    }
    Ok(converter)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let format = parse_format(&cli.format)?;
    let options = conversion_options(&cli, format);

    let output = cli
        .output
        .clone()
        .unwrap_or_else(|| default_output(&cli.input, format));

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let input_path = cli.input.clone();
    if cli.watch {
        if format != OutputFormat::Pdf {
            anyhow::bail!("Watch mode currently supports PDF output only");
        }
        let template_path = cli.template.clone();
        let force = cli.force;
        let incremental = !cli.no_incremental;
        let result: Result<(), anyhow::Error> = watch_single(&input_path, &output, move |inp, out| {
            let mut converter = StreamingConverter::with_reporter(ConsoleReporter)
                .with_incremental(incremental)
                .with_force_rebuild(force);
            if let Some(path) = &template_path {
                let template = DocumentTemplate::from_toml(path)?;
                converter = converter.with_template(template);
            }
            converter.convert(inp, out)?;
            println!("Successfully converted {} to {}", inp.display(), out.display());
            Ok(())
        });
        if let Err(e) = result {
            eprintln!("Watch mode error: {}", e);
            std::process::exit(1);
        }
    } else if format == OutputFormat::Pdf && !cli.fetch_packages {
        let mut converter = build_converter(&cli)?;
        converter.convert(&input_path, &output)?;
    } else {
        convert_tex_file(&input_path, &output, &options)?;
    }

    println!(
        "Successfully converted {} to {}",
        input_path.display(),
        output.display()
    );

    Ok(())
}
