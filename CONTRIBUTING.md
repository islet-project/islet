# Contributing to Islet

We welcome contributions to the Islet project. This document provides guidelines for contributing code, reporting issues, and maintaining coding style consistency.

For general information about the Islet project, including its architecture and setup instructions, please refer to the [official documentation](https://islet-project.github.io/islet/).

## Reporting Issues

If you'd like to report a bug or suggest an improvement, please open an [issue](https://github.com/islet-project/islet/issues).
Provide as much context as possible to help us understand the problem or proposal.

## Contributing Code

Code contributions should be made through Pull Requests (PRs).
Make sure your changes are clear, tested, and relevant to the project.

1. Fork the repository and create a new branch.
2. Make your changes and commit them.
3. Open a PR targeting the `main` branch with a clear description.

## Coding Style

Consistent formatting and linting help keep the codebase clean and reliable.
Please follow the formatting conventions:

- **Rust code**
  - Format with: `cargo fmt`
  - Lint with: `cargo clippy`
- **Shell scripts**
  - Format with: `shfmt`

These checks are included in our CI, but you can run them manually before submitting:

```sh
cargo fmt -- --check
./scripts/clippy.sh
./assets/formatter/shfmt -d -ci -bn -fn `find scripts/. -name *.sh`
```
