# Agent Guidelines for gcodekit

## Technology Stack
- **Language**: Rust version 2024 or greater. 
- **UI Framework**: egui 0.33 or greater.

## Build Commands
- `cargo build` - Build debug binary
- `cargo build --release` - Build optimized release binary
- `cargo check` - Check code without building

## Test Commands
- `cargo test` - Run all tests
- `cargo test <test_function_name>` - Run specific test function
- `cargo test -- --nocapture` - Run tests with output visible
- `cargo test --lib` - Test library only (skip integration tests)

## Lint & Format Commands
- `cargo clippy` - Run linter with clippy
- `cargo fmt` - Format code with rustfmt
- `cargo fmt --check` - Check formatting without changes

## Github access
- use "gh" to access all github repositories. 

## Documentation standards 
-  For all functions create DOCBLOCK documentation comments above each function that describes the purpose of the function, and documents any arguments and return vaulues
-  For all modules place a DOCBLOCK at the top of the File that describes the purpose of the module, and any dependancies. 

## Code Style Guidelines
- **Formatting**: 4 spaces, max 100 width, reorder_imports=true, Unix newlines
- **Naming**: snake_case for functions/variables, PascalCase for types/structs/enums
- **Imports**: Group std, external crates, then local modules; reorder automatically
- **Error Handling**: Use `Result<T, E>` with `?`, `anyhow::Result` for main, `thiserror` for custom errors
- **Types**: Prefer explicit types, use type aliases for complex types
- **Async**: Use `tokio` runtime, `async-trait` for trait methods
- **Logging**: Use `tracing` with structured logging, avoid `println!` in production
- **Documentation**: `//!` for crate docs, `///` for public APIs, `//` for internal comments
- **Linting**: No wildcard imports, cognitive complexity ≤30, warn on missing docs
- **Best Practices**: Read the best practices at https://www.djamware.com/post/68b2c7c451ce620c6f5efc56/rust-project-structure-and-best-practices-for-clean-scalable-code and apply to the project.

## Issue Handling Process
When dealing with issues in the remote repository:
1. **Analyze and Comment**: Place a comment in the issue that records your analysis of the issue and your proposed plan for fixing it
2. **Implement Fix**: After analysis, implement the proposed solution
3. **Wait for Confirmation**: Do not close the issue until the reporter has confirmed the fix is working 
