# Numerical validation

Use the Rust workspace tests as the maintained validation entry point:

```bash
cargo test --workspace
```

The former standalone reference runner has been removed so validation uses the Rust workspace only. Historical reports that were produced by that runner should not be interpreted as current Rust-source validation; they must be replaced by Rust tests before being used as release evidence.
