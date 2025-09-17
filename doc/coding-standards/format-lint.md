#  Code Formatting and Lint Checking for Islet

## Overview
This document outlines the coding standards for the Islet project, focusing on code formatting and lint checking. The project uses Rust for certain components and Bash for build and execution scripts. Below are the tools and standards employed for each language, along with instructions for running checks.

## Rust: `cargo fmt` and `cargo clippy`

We adhere to the [Official Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) and use `cargo fmt` and `cargo clippy` to ensure compliance with these standards.

### Rust Coding Standards
#### Code Formatting:
Use `cargo fmt` to adhere to the official Rust style guide, ensuring consistent code structure and readability.  

- `cargo fmt` is a Rust tool that automatically formats Rust code according to the official Rust style guide. It ensures consistent 4-space indentation, spacing, and code structure, making the codebase more readable and maintainable.

#### Lint Checking:
Use `cargo clippy` to catch common errors, improve code quality, and enforce Rust best practices.  

- `cargo clippy` is a Rust linter that identifies common programming errors, anti-patterns, and potential improvements in Rust code. It enforces best practices and helps developers write safer and more idiomatic Rust code.


### Running Checks
- To check code formatting with `cargo fmt`, navigate to the `rmm/src` directory and run:  
  ```bash
  cd rmm/src
  cargo fmt -- --check` or `cargo fmt --all
  ```
- To run `cargo clippy`, execute the script:
  (Available at: [clippy.sh](https://github.com/islet-project/islet/blob/main/scripts/clippy.sh))  
  ```bash
  scripts/tests/clippy.sh
  ```

## Bash: `shfmt`

### `shfmt`
`shfmt` is a Bash formatter that automatically formats Bash scripts to ensure consistent style and readability. It aligns code with standard Bash conventions, such as proper indentation and spacing.

### Bash Coding Standards
- **Code Formatting**: Use `shfmt` to format Bash scripts according to standard conventions, ensuring clean and readable code.
- **Lint Checking**: While `shfmt` primarily focuses on formatting, it also helps identify syntax issues and improve script structure.

### Running Checks
To verify Bash script formatting, execute the following commands:  
```bash
./assets/formatter/shfmt -d -ci -bn -fn $(find scripts/. -name *.sh)
./assets/formatter/shfmt -d -ci -bn -fn $(find examples/cross-platform-e2ee/. -name *.sh)
```
