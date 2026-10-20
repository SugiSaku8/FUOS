
OUT="sos_diagnostics.txt"

{
    echo "=================================================="
    echo " SOS - PROJECT DIAGNOSTICS"
    echo " Date: $(date)"
    echo "=================================================="

    echo
    echo "========== 1. GIT STATUS =========="
    git status --short
    git log -1 --oneline 2>/dev/null

    echo
    echo "========== 2. WORKSPACE =========="
    rustc --version
    cargo --version
    cargo metadata --no-deps --format-version 1 2>/dev/null \
        | python3 -c '
import json,sys
try:
    d=json.load(sys.stdin)
    for p in d["packages"]:
        print(f"{p[\"name\"]}: {p[\"manifest_path\"]}")
except Exception as e:
    print("Cargo metadata error:",e)
'

    echo
    echo "========== 3. SOURCE FILES =========="
    find crates -type f \( \
        -name '*.rs' -o \
        -name '*.wgsl' -o \
        -name 'Cargo.toml' \
    \) 2>/dev/null | sort

    echo
    echo "========== 4. FLOW / PARTICLE / CONTOUR REFERENCES =========="
    grep -R -n -E \
        'FlowParticleSystem|FlowGenerator|FlowVertex|GpuFlowMesh|FlowPipeline|contour_pipeline|particle_dots|velocity_pipeline|flow_pipeline' \
        crates --include='*.rs' --exclude-dir=target 2>/dev/null

    echo
    echo "========== 5. SHADER FILES =========="
    find crates -type f -name '*.wgsl' -not -path '*/target/*' \
        -print -exec sh -c '
            echo "----- $1 -----"
            sed -n "1,220p" "$1"
        ' _ {} \; 2>/dev/null

    echo
    echo "========== 6. FLOW-RELATED RUST SOURCES =========="
    find crates -type f -name '*.rs' \
        ! -path '*/target/*' \
        -print0 2>/dev/null |
    xargs -0 grep -l -E \
        'FlowParticleSystem|FlowGenerator|GpuFlowMesh|FlowPipeline|FlowVertex|VelocityField|advection' \
        2>/dev/null |
    sort -u |
    while IFS= read -r file; do
        echo
        echo "----- $file -----"
        sed -n '1,280p' "$file"
    done

    echo
    echo "========== 7. RENDERER PIPELINE / DRAW CALLS =========="
    grep -n -E \
        'create_auxiliary_pipeline|RenderPipeline::new|set_pipeline|set_vertex_buffer|draw\(|draw_indexed|DepthStencilState|depth_write_enabled|depth_compare|LoadOp::Clear|LoadOp::Load' \
        crates/s-renderer/src/renderer.rs \
        crates/s-renderer/src/flow_pipeline.rs \
        crates/s-renderer/src/velocity_pipeline.rs \
        2>/dev/null

    echo
    echo "========== 8. EXAMPLES =========="
    find crates -path '*/examples/*' -type f \
        \( -name '*.rs' -o -name '*.wgsl' \) \
        -not -path '*/target/*' 2>/dev/null | sort

    echo
    echo "========== 9. RECENT TEST RESULTS =========="
    cargo test --workspace 2>&1

    echo
    echo "========== 10. BUILD CHECK =========="
    cargo check --workspace 2>&1

    echo
    echo "========== END OF DIAGNOSTICS =========="

} > "$OUT" 2>&1

echo "診断結果を保存しました:"
echo "$PWD/$OUT"
echo
echo "ファイルサイズ:"
wc -c "$OUT"
