# SOS validation helpers

`validate_numerics.py` runs six portable manufactured/reference tests for pressure residual, projection divergence, scalar conservation, time convergence, and grid convergence.

Requirements: Python 3.10+ and NumPy.

Run:

```bash
python3 tools/validation/validate_numerics.py
```

Important: these are independent reference-operator tests, not tests of the SOS Rust or Metal implementation. See `docs/validation/VALIDATION_REPORT.md` for the result and limits.
