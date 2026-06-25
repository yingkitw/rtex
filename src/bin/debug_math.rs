use latex_rs::MathFormatter;

fn main() {
    let input = r"\int_0^\infty e^{-x} dx = 1";
    let output = MathFormatter::format(input);
    println!("Input: {}", input);
    println!("Output: {}", output);
}
