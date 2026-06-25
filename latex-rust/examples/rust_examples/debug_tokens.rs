use latex_rust::lexer::Lexer;

fn main() {
    let lexer = Lexer::new();
    let tokens = lexer.tokenize("\\title{Test Title}").unwrap();
    
    println!("Number of tokens: {}", tokens.len());
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }
}