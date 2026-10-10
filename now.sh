#!/bin/bash

echo "======================================"
echo " Suruga Ocean Simulator Status Report "
echo "======================================"

echo ""
echo "## Current directory"
pwd


echo ""
echo "======================================"
echo "1. Workspace structure"
echo "======================================"

if command -v tree >/dev/null 2>&1; then
    tree -L 3
else
    find . -maxdepth 3 -type f | sort
fi


echo ""
echo "======================================"
echo "2. Cargo workspace"
echo "======================================"

cat Cargo.toml


echo ""
echo "======================================"
echo "3. Cargo packages"
echo "======================================"

if command -v jq >/dev/null 2>&1; then
    cargo metadata --no-deps --format-version 1 \
    | jq '.packages[].name'
else
    cargo metadata --no-deps --format-version 1
fi


echo ""
echo "======================================"
echo "4. Crates"
echo "======================================"

find crates -maxdepth 2 -type d | sort


echo ""
echo "======================================"
echo "5. s-metal files"
echo "======================================"

find crates/s-metal -type f 2>/dev/null | sort


echo ""
echo "======================================"
echo "6. Metal Operators"
echo "======================================"

grep -R "struct .*Operator" crates/s-metal/src 2>/dev/null


echo ""
echo "======================================"
echo "7. OceanSolver"
echo "======================================"

sed -n '1,260p' crates/s-metal/src/solver.rs 2>/dev/null


echo ""
echo "======================================"
echo "8. Metal shaders"
echo "======================================"

find crates/s-metal -name "*.metal" -o -name "*.metalh" 2>/dev/null


echo ""
echo "======================================"
echo "9. Metal kernel functions"
echo "======================================"

grep -R "kernel void" crates/s-metal 2>/dev/null


echo ""
echo "======================================"
echo "10. Metal dispatch calls"
echo "======================================"

grep -R "dispatch" crates/s-metal/src 2>/dev/null


echo ""
echo "======================================"
echo "11. Simulation files"
echo "======================================"

find crates/simulation -type f 2>/dev/null | sort


echo ""
echo "======================================"
echo "12. Simulation lib"
echo "======================================"

sed -n '1,260p' crates/simulation/src/lib.rs 2>/dev/null


echo ""
echo "======================================"
echo "13. Examples"
echo "======================================"

find crates -path "*examples*" -type f -name "*.rs" 2>/dev/null


echo ""
echo "======================================"
echo "14. s-field structures"
echo "======================================"

grep -R "pub struct" crates/s-field/src 2>/dev/null


echo ""
echo "======================================"
echo "15. OceanField"
echo "======================================"

sed -n '1,260p' crates/s-field/src/ocean.rs 2>/dev/null


echo ""
echo "======================================"
echo "16. s-sof files"
echo "======================================"

find crates/s-sof/src -type f 2>/dev/null | sort


echo ""
echo "======================================"
echo "17. s-sof API"
echo "======================================"

grep -R "pub struct\|pub fn" crates/s-sof/src 2>/dev/null


echo ""
echo "======================================"
echo "18. Numerical operators usage"
echo "======================================"

echo "--- Advection ---"
grep -R "Advection" crates 2>/dev/null

echo ""
echo "--- Diffusion ---"
grep -R "Diffusion" crates 2>/dev/null

echo ""
echo "--- Divergence ---"
grep -R "Divergence" crates 2>/dev/null

echo ""
echo "--- Pressure ---"
grep -R "Pressure" crates 2>/dev/null


echo ""
echo "======================================"
echo "19. TODO / unfinished"
echo "======================================"

grep -R "TODO\|todo!\|unimplemented!\|IMPLEMENT" crates 2>/dev/null


echo ""
echo "======================================"
echo "20. Build status"
echo "======================================"

cargo check --workspace


echo ""
echo "======================================"
echo "21. Tests"
echo "======================================"

cargo test --workspace -- --list



echo ""
echo "======================================"
echo " DONE "
echo "======================================"
