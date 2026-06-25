use criterion::{black_box, criterion_group, criterion_main, Criterion};
use latex_rust::lexer::Lexer;

fn bench_tokenize_simple_text(c: &mut Criterion) {
    let input = "This is a simple text document with no special formatting.";
    
    c.bench_function("tokenize_simple_text", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            lexer.tokenize(black_box(input)).unwrap()
        })
    });
}

fn bench_tokenize_with_commands(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \usepackage{amsmath}
        \title{Test Document}
        \author{Test Author}
        \begin{document}
        \maketitle
        \section{Introduction}
        This is a test document with \textbf{bold} and \textit{italic} text.
        \end{document}
    "#;
    
    c.bench_function("tokenize_with_commands", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            lexer.tokenize(black_box(input)).unwrap()
        })
    });
}

fn bench_tokenize_math_heavy(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \usepackage{amsmath}
        \begin{document}
        \section{Mathematics}
        Here are some equations: $E = mc^2$ and $x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$.
        
        Display math:
        $$\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}$$
        
        $$\sum_{n=1}^\infty \frac{1}{n^2} = \frac{\pi^2}{6}$$
        
        More inline math: $\alpha + \beta = \gamma$ and $\lim_{x \to 0} \frac{\sin x}{x} = 1$.
        \end{document}
    "#;
    
    c.bench_function("tokenize_math_heavy", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            lexer.tokenize(black_box(input)).unwrap()
        })
    });
}

fn bench_tokenize_large_document(c: &mut Criterion) {
    // Generate a large document
    let mut input = String::from("\\documentclass{article}\n\\begin{document}\n");
    
    for i in 0..1000 {
        input.push_str(&format!(
            "\\section{{Section {}}}\nThis is section {} with some \\textbf{{bold}} text and $x_{{{}}} = {}$.\n",
            i, i, i, i
        ));
    }
    
    input.push_str("\\end{document}");
    
    c.bench_function("tokenize_large_document", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            lexer.tokenize(black_box(&input)).unwrap()
        })
    });
}

fn bench_tokenize_nested_environments(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \begin{document}
        \begin{itemize}
        \item First item
        \item Second item with \textbf{bold} text
        \begin{enumerate}
            \item Nested item 1
            \item Nested item 2 with $math = formula$
            \begin{itemize}
                \item Deeply nested item
                \item Another deeply nested item
            \end{itemize}
        \end{enumerate}
        \item Third item
        \end{itemize}
        \end{document}
    "#;
    
    c.bench_function("tokenize_nested_environments", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            lexer.tokenize(black_box(input)).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_tokenize_simple_text,
    bench_tokenize_with_commands,
    bench_tokenize_math_heavy,
    bench_tokenize_large_document,
    bench_tokenize_nested_environments
);
criterion_main!(benches);