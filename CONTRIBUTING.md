# Contributing to ArcLang

Thank you for your interest in contributing to ArcLang! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please be respectful and professional in all interactions.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Create a branch** for your feature or bugfix
4. **Make your changes** and commit them
5. **Push to your fork** and submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git 2.30 or later
- A C compiler (for some dependencies)

### Building from Source

```bash
git clone https://github.com/arclang/arclang.git
cd arclang
cargo build
cargo test
```

### Running Tests

```bash
# Run all tests
cargo test --all-features --workspace

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_tests
```

### Code Formatting

```bash
# Check formatting
cargo fmt --all -- --check

# Format code
cargo fmt --all
```

### Linting

```bash
# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

## Contribution Guidelines

### Branching Strategy

- `main`: Stable, production-ready code
- `develop`: Integration branch for features
- `feature/*`: Feature branches
- `bugfix/*`: Bug fix branches
- `release/*`: Release preparation branches

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(compiler): add incremental compilation support

fix(plm): correct Windchill sync timeout issue

docs(api): update plugin development guide
```

### Pull Request Process

1. **Update documentation** if you're changing functionality
2. **Add tests** for new features or bug fixes
3. **Ensure all tests pass**: `cargo test --all-features`
4. **Run formatting and linting**: `cargo fmt && cargo clippy`
5. **Update CHANGELOG.md** with your changes
6. **Create a pull request** with a clear title and description

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Documentation updated
- [ ] Tests pass
- [ ] CHANGELOG.md updated
```

## Code Style Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting (config in `.rustfmt.toml`)
- Maximum line length: 100 characters
- Use meaningful variable and function names
- Add documentation comments for public APIs

### Documentation Comments

```rust
/// Brief description of the function
///
/// # Arguments
///
/// * `param1` - Description of param1
/// * `param2` - Description of param2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of possible errors
///
/// # Examples
///
/// ```
/// let result = my_function(arg1, arg2)?;
/// ```
pub fn my_function(param1: Type1, param2: Type2) -> Result<ReturnType, Error> {
    // Implementation
}
```

### Error Handling

- Use `Result<T, E>` for fallible operations
- Create specific error types using `thiserror`
- Provide helpful error messages
- Use `?` operator for error propagation

```rust
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Failed to compile: {0}")]
    CompilationError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Testing

- Write unit tests for individual functions
- Write integration tests for end-to-end scenarios
- Use descriptive test names
- Test edge cases and error conditions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        let result = my_function(input);
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_error_handling() {
        let result = my_function(invalid_input);
        assert!(result.is_err());
    }
}
```

## Areas for Contribution

### High Priority

- **Performance optimization**: Improve compilation speed
- **PLM integration**: Add support for new PLM systems
- **Safety analysis**: Enhance FMEA/FTA algorithms
- **Documentation**: Improve user guides and examples
- **Bug fixes**: Address open issues

### Medium Priority

- **Plugin development**: Create useful plugins
- **Code generators**: Add new target languages
- **IDE integration**: Improve language server
- **Testing**: Increase test coverage
- **Examples**: Add more real-world examples

### Low Priority

- **Visualization**: Improve diagram generation
- **UI/UX**: CLI improvements
- **Tooling**: Developer experience enhancements

## Specific Contribution Areas

### Adding a New PLM Connector

1. Create a new file in `plm/connectors/`
2. Implement the `PLMConnector` trait
3. Add configuration in `plm/config.rs`
4. Add tests in `plm/connectors/tests/`
5. Update documentation

### Creating a Plugin

1. Create a new crate in `plugins/` or external repository
2. Implement the `Plugin` trait
3. Add a `plugin.toml` manifest
4. Document plugin capabilities
5. Submit plugin to the registry

### Improving Safety Analysis

1. Review existing algorithms in `safety/`
2. Propose improvements with rationale
3. Implement with comprehensive tests
4. Validate against safety standards
5. Update compliance documentation

## Documentation

### User Documentation

- Located in `docs/`
- Written in Markdown
- Include code examples
- Keep up-to-date with code changes

### API Documentation

- Use Rust doc comments (`///`)
- Include examples in documentation
- Document all public APIs
- Generate with `cargo doc`

## Testing Guidelines

### Unit Tests

- Test individual functions in isolation
- Mock external dependencies
- Cover edge cases and error paths
- Use `#[cfg(test)]` modules

### Integration Tests

- Test end-to-end workflows
- Use realistic data
- Test interaction between components
- Located in `tests/` directory

### Performance Tests

- Use `criterion` for benchmarks
- Test with realistic data sizes
- Compare before and after changes
- Document performance requirements

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release branch: `release/vX.Y.Z`
4. Run full test suite
5. Create tag: `git tag vX.Y.Z`
6. Push tag: `git push origin vX.Y.Z`
7. CI will build and publish release

## Getting Help

- **GitHub Discussions**: Ask questions and share ideas
- **GitHub Issues**: Report bugs or request features
- **Slack**: Join our Slack workspace for real-time chat
- **Email**: contact@arclang.org

## Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Project README

Thank you for contributing to ArcLang!
