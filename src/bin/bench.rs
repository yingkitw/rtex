use std::path::Path;
use std::time::Instant;

fn main() {
    let examples = [
        "examples/minimal.tex",
        "examples/math.tex",
        "examples/sample.tex",
        "examples/lists.tex",
        "examples/table.tex",
        "examples/code.tex",
    ];

    let iterations = 10;
    let mut total_avg = 0.0;
    let mut count = 0;

    println!("Benchmarking {} iterations per file...\n", iterations);

    for example in &examples {
        let input = Path::new(example);
        if !input.exists() {
            println!("SKIP: {} (not found)", example);
            continue;
        }

        let mut times = Vec::new();
        for i in 0..iterations {
            let output_path = format!("output/bench_{}_{}.pdf", example.replace('/', "_"), i);
            let output = Path::new(&output_path);
            let start = Instant::now();
            let _ = latex_rs::convert_tex_to_pdf(input, output);
            times.push(start.elapsed().as_millis() as f64);
        }

        let avg = times.iter().sum::<f64>() / times.len() as f64;
        let min = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = times.iter().fold(0.0_f64, |a, &b| a.max(b));
        total_avg += avg;
        count += 1;

        println!(
            "{:25} | avg {:>7.1} ms | min {:>7.1} ms | max {:>7.1} ms",
            example, avg, min, max
        );
    }

    if count > 0 {
        println!("\nOverall average: {:.1} ms per conversion", total_avg / count as f64);
    }
}
