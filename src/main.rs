use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use rtex::{ConsoleReporter, ConversionOptions, LatexError, OutputFormat, StreamingConverter, watch_single};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "rtex")]
#[command(about = "Convert TeX files to PDF, HTML, DOCX, or EPUB", long_about = None)]
struct Cli {
    #[arg(help = "Input TeX file path")]
    input: Option<PathBuf>,

    #[arg(short, long, help = "Output file path (ignored when multiple formats are requested)")]
    output: Option<PathBuf>,

    #[arg(
        short,
        long,
        value_name = "FORMAT",
        value_delimiter = ',',
        default_value = "pdf",
        help = "Output format(s): pdf, html, docx, epub. Comma-separated for multiple outputs."
    )]
    format: Vec<String>,

    #[arg(
        long,
        help = "Fetch missing LaTeX packages from CTAN before conversion"
    )]
    fetch_packages: bool,

    #[arg(long, help = "Directory for downloaded LaTeX package cache")]
    package_cache: Option<PathBuf>,

    #[arg(short, long, help = "Watch input file and recompile on changes")]
    watch: bool,

    #[arg(
        long,
        help = "Force rebuild even when source and dependencies are unchanged"
    )]
    force: bool,

    #[arg(long, help = "Disable incremental compilation and always rebuild")]
    no_incremental: bool,

    #[arg(
        long,
        help = "Keep intermediate files (.expanded.tex, .ast.json, .meta.json)"
    )]
    keep_intermediate: bool,

    #[arg(
        long,
        value_name = "SHELL",
        help = "Generate shell completion script (bash, zsh, fish, elvish, powershell)"
    )]
    completions: Option<Shell>,
}

fn parse_formats(raw: &[String]) -> Result<Vec<OutputFormat>, LatexError> {
    raw.iter().map(|s| s.parse()).collect()
}

fn default_output(input: &Path, format: OutputFormat) -> PathBuf {
    let output_dir = PathBuf::from("output");
    fs::create_dir_all(&output_dir).ok();
    let filename = input
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    output_dir.join(format!("{}.{}", filename, format.extension()))
}

fn build_options(cli: &Cli, format: OutputFormat) -> ConversionOptions {
    ConversionOptions {
        format,
        fetch_packages: cli.fetch_packages,
        package_cache: cli.package_cache.clone(),
        keep_intermediate: cli.keep_intermediate,
    }
}

fn build_converter(
    cli: &Cli,
    options: ConversionOptions,
) -> StreamingConverter<ConsoleReporter> {
    StreamingConverter::with_reporter(ConsoleReporter)
        .with_options(options)
        .with_incremental(!cli.no_incremental)
        .with_force_rebuild(cli.force)
        .with_keep_intermediate(cli.keep_intermediate)
}

fn derive_output(cli: &Cli, input: &Path, format: OutputFormat) -> PathBuf {
    if cli.format.len() > 1 {
        if cli.output.is_some() {
            eprintln!("note: -o/--output is ignored when multiple formats are requested");
        }
        default_output(input, format)
    } else {
        cli.output
            .clone()
            .unwrap_or_else(|| default_output(input, format))
    }
}

fn convert_one(
    cli: &Cli,
    input: &Path,
    output: &Path,
    format: OutputFormat,
) -> Result<(), LatexError> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut converter = build_converter(cli, build_options(cli, format));
    converter.convert(input, output)?;
    println!("Successfully converted {} to {}", input.display(), output.display());
    Ok(())
}

fn convert_all(cli: &Cli, input: &Path, formats: &[OutputFormat]) -> Result<(), LatexError> {
    for &format in formats {
        let output = derive_output(cli, input, format);
        convert_one(cli, input, &output, format)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "rtex", &mut std::io::stdout());
        return Ok(());
    }

    let input_path = cli.input.clone().ok_or_else(|| {
        LatexError::ConfigError {
            message: "input file path is required (use --completions SHELL to generate completions instead)".to_string(),
        }
    })?;

    let formats = parse_formats(&cli.format)?;

    if cli.watch {
        if formats.len() > 1 {
            eprintln!("note: watch mode emits all requested formats on each rebuild");
        }
        let force = cli.force;
        let incremental = !cli.no_incremental;
        let keep_intermediate = cli.keep_intermediate;
        let fetch_packages = cli.fetch_packages;
        let package_cache = cli.package_cache.clone();
        let formats_for_loop = formats.clone();
        let stem = input_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let watch_target = derive_output(&cli, &input_path, formats[0]);
        let result: Result<(), LatexError> =
            watch_single(&input_path, &watch_target, move |inp, _out| {
                for &fmt in &formats_for_loop {
                    let out = PathBuf::from("output").join(format!("{stem}.{}", fmt.extension()));
                    if let Some(parent) = out.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let options = ConversionOptions {
                        format: fmt,
                        fetch_packages,
                        package_cache: package_cache.clone(),
                        keep_intermediate,
                    };
                    let mut converter = StreamingConverter::with_reporter(ConsoleReporter)
                        .with_options(options)
                        .with_incremental(incremental)
                        .with_force_rebuild(force)
                        .with_keep_intermediate(keep_intermediate);
                    converter.convert(inp, &out)?;
                    println!("Successfully converted {} to {}", inp.display(), out.display());
                }
                Ok(())
            });
        if let Err(e) = result {
            eprintln!("Watch mode error: {}", e);
            std::process::exit(1);
        }
    } else {
        convert_all(&cli, &input_path, &formats)?;
    }

    Ok(())
}