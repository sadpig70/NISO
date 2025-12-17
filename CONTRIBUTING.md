# Contributing to NISO

Thank you for your interest in contributing to NISO (NISQ Integrated System Optimizer)! We welcome contributions from the quantum computing community.

## How to Contribute

### 1. Reporting Bugs

- Please check existing issues on GitHub to avoid duplicates.
- Provide a clear title and description.
- Include steps to reproduce, expected behavior, and actual behavior.
- Include logs or error messages if applicable.

### 2. Suggesting Enhancements

- Explain the rationale and use case for the enhancement.
- If possible, provide a design sketch or pseudo-code.

### 3. Pull Requests

1. **Fork the repository** on GitHub.
2. **Create a branch** for your feature or fix (`git checkout -b feature/amazing-feature`).
3. **Commit your changes** using [Conventional Commits](https://www.conventionalcommits.org/).
4. **Run tests** to ensure no regressions (`cargo test`).
5. **Format your code** (`cargo fmt`).
6. **Run clippy** (`cargo clippy`) and address warnings.
7. **Push to your fork** (`git push origin feature/amazing-feature`).
8. **Open a Pull Request** against the `main` branch.

## Coding Standards

- **Rust**: We follow standard Rust formatting (`rustfmt`) and idiomatic practices (`clippy`).
- **Documentation**: All public APIs must have documentation strings (`///`).
- **Testing**: New features must include unit tests. Integration tests are encouraged for major changes.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
