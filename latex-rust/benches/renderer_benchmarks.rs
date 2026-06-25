use criterion::{black_box, criterion_group, criterion_main, Criterion};
use latex_rust::lexer::Lexer;
use latex_rust::parser::Parser;
use latex_rust::renderer::HtmlRenderer;

fn bench_render_simple_document(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \begin{document}
        \section{Introduction}
        This is a simple document with basic text formatting.
        \end{document}
    "#;
    
    // Pre-parse the document
    let lexer = Lexer::new();
    let tokens = lexer.tokenize(input).unwrap();
    let mut parser = Parser::new();
    let document = parser.parse(tokens).unwrap();
    
    c.bench_function("render_simple_document", |b| {
        b.iter(|| {
            let renderer = HtmlRenderer::new();
            renderer.render(black_box(&document))
        })
    });
}

fn bench_render_with_formatting(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \begin{document}
        \section{Text Formatting}
        This document contains \textbf{bold}, \textit{italic}, \texttt{monospace}, 
        \underline{underlined}, and \emph{emphasized} text formatting.
        \subsection{More Formatting}
        Additional text with various formatting options.
        \end{document}
    "#;
    
    let lexer = Lexer::new();
    let tokens = lexer.tokenize(input).unwrap();
    let mut parser = Parser::new();
    let document = parser.parse(tokens).unwrap();
    
    c.bench_function("render_with_formatting", |b| {
        b.iter(|| {
            let renderer = HtmlRenderer::new();
            renderer.render(black_box(&document))
        })
    });
}

fn bench_render_math_heavy(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \usepackage{amsmath}
        \begin{document}
        \section{Mathematics}
        Inline math: $E = mc^2$ and $x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$.
        
        Display math:
        $$\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}$$
        
        $$\sum_{n=1}^\infty \frac{1}{n^2} = \frac{\pi^2}{6}$$
        
        More equations: $\alpha + \beta = \gamma$ and $\lim_{x \to 0} \frac{\sin x}{x} = 1$.
        \end{document}
    "#;
    
    let lexer = Lexer::new();
    let tokens = lexer.tokenize(input).unwrap();
    let mut parser = Parser::new();
    let document = parser.parse(tokens).unwrap();
    
    c.bench_function("render_math_heavy", |b| {
        b.iter(|| {
            let renderer = HtmlRenderer::new();
            renderer.render(black_box(&document))
        })
    });
}

fn bench_render_lists_and_environments(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \begin{document}
        \section{Lists and Environments}
        \begin{itemize}
        \item First item
        \item Second item with \textbf{bold} text
        \item Third item
        \end{itemize}
        
        \begin{enumerate}
        \item Numbered item one
        \item Numbered item two
        \item Numbered item three
        \end{enumerate}
        
        \begin{quote}
        This is a quoted section with some text.
        \end{quote}
        
        \begin{center}
        This text is centered.
        \end{center}
        \end{document}
    "#;
    
    let lexer = Lexer::new();
    let tokens = lexer.tokenize(input).unwrap();
    let mut parser = Parser::new();
    let document = parser.parse(tokens).unwrap();
    
    c.bench_function("render_lists_and_environments", |b| {
        b.iter(|| {
            let renderer = HtmlRenderer::new();
            renderer.render(black_box(&document))
        })
    });
}

fn bench_render_complex_document(c: &mut Criterion) {
    let input = r#"
        \documentclass{article}
        \usepackage{amsmath}
        \usepackage{graphicx}
        \title{Complex Document}
        \author{Benchmark Test}
        \begin{document}
        \maketitle
        \begin{abstract}
        This is a complex document with multiple features including sections,
        mathematics, lists, and various text formatting.
        \end{abstract}
        
        \section{Introduction}
        This section contains various elements and formatting.
        
        \subsection{Text Formatting}
        Here we have \textbf{bold}, \textit{italic}, \texttt{monospace}, 
        \underline{underlined}, and \emph{emphasized} text.
        
        \subsection{Mathematics}
        Some inline math: $\alpha + \beta = \gamma$ and display math:
        $$\int_0^1 x^2 dx = \frac{1}{3}$$
        
        \section{Lists and Environments}
        \begin{itemize}
        \item Item one with $math = formula$
        \item Item two with \textbf{bold formatting}
        \begin{enumerate}
            \item Nested numbered item
            \item Another nested item
        \end{enumerate}
        \item Item three
        \end{itemize}
        
        \begin{quote}
        This is a quoted section that demonstrates the quote environment
        with some longer text content.
        \end{quote}
        
        \section{Conclusion}
        This concludes our complex document example.
        \end{document}
    "#;
    
    let lexer = Lexer::new();
    let tokens = lexer.tokenize(input).unwrap();
    let mut parser = Parser::new();
    let document = parser.parse(tokens).unwrap();
    
    c.bench_function("render_complex_document", |b| {
        b.iter(|| {
            let renderer = HtmlRenderer::new();
            renderer.render(black_box(&document))
        })
    });
}

criterion_group!(
    benches,
    bench_render_simple_document,
    bench_render_with_formatting,
    bench_render_math_heavy,
    bench_render_lists_and_environments,
    bench_render_complex_document
);
criterion_main!(benches);