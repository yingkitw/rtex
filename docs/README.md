# rtex Documentation

| Document | Audience | Description |
|----------|----------|-------------|
| [../README.md](../README.md) | Everyone | Quick start, features, CLI |
| [LSP.md](LSP.md) | Users / Editors | Language server setup and capabilities |
| [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md) | Users | What rtex cannot do |
| [../ARCHITECTURE.md](../ARCHITECTURE.md) | Contributors | Module design and data flow |
| [../SPEC.md](../SPEC.md) | Contributors | Scope, stack, quality bar |
| [../TODO.md](../TODO.md) | Contributors | Task backlog |
| [TESTING.md](TESTING.md) | Contributors | Test strategy |
| [TRAIT_ARCHITECTURE.md](TRAIT_ARCHITECTURE.md) | Contributors | Trait-based design |

## API Reference

Generate rustdoc locally:

```bash
cargo doc --open --no-deps
```

Key public types: `NativeTexConverter`, `OutputFormat`, `ConversionOptions`, `PackageFetcher`, `TexParser`, `TexElement`, `PdfBuilder`.

## Historical Notes

Files in `docs/historical/` describe earlier migrations (lopdf removal, refactoring) and may be outdated.
