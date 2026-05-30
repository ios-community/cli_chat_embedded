# Contributing to cli_chat_embedded

First off, thank you for considering contributing to `cli_chat_embedded`! It's people like you that make open source such a great community.

## Development Process

This project follows **Spec Driven Development (SDD)**. All new features, architectural changes, or memory layout modifications must be discussed and specified in an issue before implementation.

### 1. Core Constraints
- **Zero-Heap** \
The `core` library must remain strictly `#![no_std]`. Do not use the `alloc` crate, `Vec`, `String`, `Box`, or any heap-allocated structures.
- **No Panics** \
The core library should never panic under normal operation. Return `Result` with static string slices (`&'static str`).
- **Unsafe Code** \
`unsafe` is globally allowed (`#![allow(unsafe_code)]`).

### 2. Documentation Standard
We enforce a strict 7-part rustdoc structure for all `pub` items. Your pull request will not be accepted if it lacks any of these sections:
1. **Summary**: One-sentence description.
2. **Description**: Detailed context and behavior.
3. **Examples**: Runnable doctest.
4. **Panics**: Conditions that cause a panic (if any).
5. **Errors**: When `Result::Err` is returned.
6. **Safety**: Invariants for `unsafe` blocks (if any).
7. **See Also**: Links to related items.

### 3. Pull Request Process
Before submitting a PR, ensure your code passes the strict validation pipeline:

```bash
# 1. Check formatting
cargo fmt --all -- --check

# 2. Lint with Clippy (Pedantic)
cargo clippy --all-features -- -D warnings

# 3. Run tests and fuzzing
cargo test --all-features

# 4. Run benchmarks (ensure no performance regression)
cargo bench --features std

# 5. Validate documentation
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

1. Fork the repository and create your branch from `main`.
2. If you've added code that should be tested, add tests (including `proptest` if applicable).
3. Ensure the test suite passes locally.
4. Issue that pull request!
