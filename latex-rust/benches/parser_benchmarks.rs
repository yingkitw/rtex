use criterion::{black_box, criterion_group, criterion_main, Criterion};
use latex_rust::lexer::Lexer;
use latex_rust::parser::Parser;

fn bench_parse_simple_document(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \begin{document}
        \section{Introduction}
        This is a simple document with basic text formatting.
        \end{document}
    "#;
    
    c.bench_function("parse_simple_document", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            let tokens = lexer.tokenize(black_box(input)).unwrap();
            let mut parser = Parser::new();
            parser.parse(tokens).unwrap()
        })
    });
}

fn bench_parse_with_commands(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \usepackage{amsmath}
        \title{Test Document}
        \author{Test Author}
        \begin{document}
        \maketitle
        \section{Introduction}
        This document contains \textbf{bold}, \textit{italic}, and \texttt{monospace} text.
        \subsection{Subsection}
        More content with \emph{emphasis} and \underline{underlined} text.
        \end{document}
    "#;
    
    c.bench_function("parse_with_commands", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            let tokens = lexer.tokenize(black_box(input)).unwrap();
            let mut parser = Parser::new();
            parser.parse(tokens).unwrap()
        })
    });
}

fn bench_parse_math_expressions(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \usepackage{amsmath}
        \begin{document}
        \section{Mathematics}
        Inline math: $E = mc^2$ and $x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$.
        
        Display math:
        $$\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}$$
        
        $$\sum_{n=1}^\infty \frac{1}{n^2} = \frac{\pi^2}{6}$$
        \end{document}
    "#;
    
    c.bench_function("parse_math_expressions", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            let tokens = lexer.tokenize(black_box(input)).unwrap();
            let mut parser = Parser::new();
            parser.parse(tokens).unwrap()
        })
    });
}

fn bench_parse_nested_environments(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \begin{document}
        \section{Lists}
        \begin{itemize}
        \item First item
        \item Second item
        \begin{enumerate}
            \item Nested numbered item
            \item Another numbered item
            \begin{itemize}
                \item Deeply nested item
                \item Another deeply nested item
            \end{itemize}
        \end{enumerate}
        \item Third item
        \end{itemize}
        \end{document}
    "#;
    
    c.bench_function("parse_nested_environments", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            let tokens = lexer.tokenize(black_box(input)).unwrap();
            let mut parser = Parser::new();
            parser.parse(tokens).unwrap()
        })
    });
}

fn bench_parse_complex_document(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \usepackage{amsmath}
        \usepackage{graphicx}
        \title{Complex Document}
        \author{Benchmark Test}
        \begin{document}
        \maketitle
        \begin{abstract}
        This is a complex document with multiple features.
        \end{abstract}
        \section{Introduction}
        This section contains various elements.
        \subsection{Text Formatting}
        Here we have \textbf{bold}, \textit{italic}, and \texttt{monospace} text.
        \subsection{Mathematics}
        Some math: $\alpha + \beta = \gamma$ and display math:
        $$\int_0^1 x^2 dx = \frac{1}{3}$$
        \section{Lists and Environments}
        \begin{itemize}
        \item Item one
        \item Item two with $math = formula$
        \end{itemize}
        \begin{quote}
        This is a quoted section.
        \end{quote}
        \end{document}
    "#;
    
    c.bench_function("parse_complex_document", |b| {
        b.iter(|| {
            let lexer = Lexer::new();
            let tokens = lexer.tokenize(black_box(input)).unwrap();
            let mut parser = Parser::new();
            parser.parse(tokens).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_parse_simple_document,
    bench_parse_with_commands,
    bench_parse_math_expressions,
    bench_parse_nested_environments,
    bench_parse_complex_document
);
criterion_main!(benches);