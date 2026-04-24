use latex_rs::math_formatter::MathFormatter;

fn main() {
    let input = r"\int_0^\infty e^{-x} dx = 1";
    let output = MathFormatter::format(input);
    println!("Input: {}", input);
    println!("Output: {}", output);
}
