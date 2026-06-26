# Trait-Based Architecture

## Overview

rtex uses a trait-based architecture for modularity, testability, and extensibility. This design is inspired by minitex's atomic traits pattern.

## Core Traits

### 1. `TexParser`
Parses LaTeX content into structured elements.

```rust
pub trait TexParser: Send + Sync {
    fn parse(&self, content: &str) -> Result<Vec<TexElement>>;
    fn parse_command(&self, content: &str, position: usize) -> Result<Option<TexElement>>;
}
```

**Benefits:**
- Easy to test with mock implementations
- Can swap parsers for different LaTeX dialects
- Enables parallel parsing

### 2. `MathFormatter`
Formats mathematical expressions from LaTeX to display format.

```rust
pub trait MathFormatter: Send + Sync {
    fn format(&self, math: &str) -> String;
    fn format_inline(&self, math: &str) -> String;
    fn format_display(&self, math: &str) -> String;
}
```

**Benefits:**
- Pluggable math renderers
- Easy to add new symbol sets
- Testable in isolation

### 3. `PdfBuilder`
Builds PDF documents from parsed elements.

```rust
pub trait PdfBuilder: Send + Sync {
    fn build(&mut self, elements: Vec<TexElement>, output_path: &Path) -> Result<()>;
    fn set_title(&mut self, title: String);
    fn set_author(&mut self, author: String);
    fn set_date(&mut self, date: String);
}
```

**Benefits:**
- Can swap PDF libraries
- Easy to test without generating files
- Supports different output formats

### 4. `FontProvider`
Manages font loading and character support.

```rust
pub trait FontProvider: Send + Sync {
    fn load_font(&self, path: &Path) -> Result<Vec<u8>>;
    fn default_font(&self) -> Result<Vec<u8>>;
    fn supports_character(&self, font_data: &[u8], ch: char) -> bool;
}
```

**Benefits:**
- Pluggable font sources
- Easy to test font fallback
- Supports custom fonts

### 5. `Cache<K, V>`
Generic caching operations.

```rust
pub trait Cache<K, V>: Send + Sync {
    fn get(&self, key: &K) -> Option<&V>;
    fn insert(&mut self, key: K, value: V);
    fn clear(&mut self);
    fn size(&self) -> usize;
}
```

**Benefits:**
- Pluggable cache implementations
- Easy to test with mock cache
- Performance optimization

### 6. `TextLayout`
Text layout and wrapping operations.

```rust
pub trait TextLayout: Send + Sync {
    fn wrap_text(&self, text: &str, max_width: usize) -> Vec<String>;
    fn text_width(&self, text: &str) -> f32;
    fn line_height(&self, font_size: f32) -> f32;
}
```

**Benefits:**
- Different layout algorithms
- Easy to test layout logic
- Supports multiple languages

## Design Principles

### 1. Single Responsibility
Each trait has one clear purpose.

### 2. Composability
Traits can be combined to build features.

```rust
struct DocumentProcessor<P, F, B>
where
    P: TexParser,
    F: MathFormatter,
    B: PdfBuilder,
{
    parser: P,
    formatter: F,
    builder: B,
}
```

### 3. Dependency Injection
Dependencies are passed as trait objects or generics.

```rust
fn process_document<P: TexParser>(parser: &P, content: &str) -> Result<Vec<TexElement>> {
    parser.parse(content)
}
```

### 4. Thread Safety
All traits require `Send + Sync` for parallel processing.

### 5. Testability
Easy to create mock implementations for testing.

```rust
struct MockMathFormatter;

impl MathFormatter for MockMathFormatter {
    fn format(&self, math: &str) -> String {
        format!("[MATH: {}]", math)
    }
}
```

## Usage Examples

### Using Traits for Testing

```rust
#[test]
fn test_with_mock_formatter() {
    let formatter = MockMathFormatter;
    let result = formatter.format("\\alpha");
    assert_eq!(result, "[MATH: \\alpha]");
}
```

### Dependency Injection

```rust
fn convert_document<P, F, B>(
    parser: &P,
    formatter: &F,
    builder: &mut B,
    content: &str,
    output: &Path,
) -> Result<()>
where
    P: TexParser,
    F: MathFormatter,
    B: PdfBuilder,
{
    let elements = parser.parse(content)?;
    builder.build(elements, output)?;
    Ok(())
}
```

### Trait Objects for Runtime Polymorphism

```rust
fn process_with_any_formatter(formatter: &dyn MathFormatter, math: &str) -> String {
    formatter.format(math)
}
```

## Current Implementations

### MathFormatter
- `MathFormatter` struct implements the trait
- Supports 150+ mathematical symbols
- Unicode output

### Future Implementations

#### TexParser
- `NativeTexParser` - Current implementation
- `StrictTexParser` - Strict LaTeX compliance
- `MarkdownTexParser` - Markdown with LaTeX math

#### PdfBuilder
- `LopdfBuilder` - Current implementation
- `PrintpdfBuilder` - Alternative backend
- `HtmlBuilder` - HTML output

#### Cache
- `MemoryCache` - In-memory caching
- `FileCache` - Persistent file cache
- `NoOpCache` - Disabled caching

## Benefits

### Modularity
- Clear separation of concerns
- Easy to understand and maintain
- Independent development of components

### Testability
- Mock implementations for unit tests
- Test components in isolation
- Fast test execution

### Extensibility
- Add new implementations without changing existing code
- Plugin system support
- Easy to experiment with alternatives

### Performance
- Thread-safe by design
- Enables parallel processing
- Efficient resource management

## Migration Path

### Phase 1: Define Traits ✅
- Created `src/traits.rs`
- Defined core traits
- Added documentation

### Phase 2: Implement Traits (In Progress)
- ✅ MathFormatter trait implementation
- ⏳ TexParser trait implementation
- ⏳ PdfBuilder trait implementation

### Phase 3: Refactor Code
- Use trait bounds in functions
- Replace concrete types with trait objects
- Add dependency injection

### Phase 4: Add Implementations
- Alternative parsers
- Alternative builders
- Caching implementations

## Best Practices

1. **Keep traits small** - Single responsibility
2. **Use trait bounds** - Generic functions with trait constraints
3. **Provide default implementations** - Where sensible
4. **Document trait contracts** - Clear expectations
5. **Test trait implementations** - Comprehensive test coverage

## References

- MiniTeX atomic traits: `src/core/traits.rs`
- Rust trait documentation
- Design patterns for trait-based architecture
