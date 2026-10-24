# Validation report

The maintained validation command is `cargo test --workspace`. Rust tests and the simulator build must be run on the target macOS environment before a release. No standalone scripting-language validation runner is part of the project.

Previous standalone reference-runner results are not evidence that the current Rust/Metal implementation compiles or passes tests. Regenerate release evidence from the current Rust workspace.
