//! Compatibility re-export. The shared EOS implementation lives in `physics` so
//! both the CPU reference solver and Metal solver use exactly the same equation.
pub use physics::equation_of_state::*;
