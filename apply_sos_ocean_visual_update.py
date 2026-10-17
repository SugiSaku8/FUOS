#!/usr/bin/env python3
"""Apply the SOS ocean/current visualization update without replacing renderer.rs.

Run from any directory inside a workspace that contains crates/s-renderer/src.
The script validates all required targets before writing and makes timestamped backups.
It is intentionally idempotent and preserves unrelated local renderer code, including
an existing bathymetry contour pipeline.
"""
from __future__ import annotations

import datetime as _dt
import re
import sys
from pathlib import Path

FILES = {
    "renderer": Path("crates/s-renderer/src/renderer.rs"),
    "flow_generator": Path("crates/s-renderer/src/flow_generator.rs"),
    "example": Path("crates/s-renderer/examples/terrain.rs"),
    "flow_shader": Path("crates/s-renderer/src/flow.wgsl"),
    "particle_shader": Path("crates/s-renderer/src/particle.wgsl"),
    "ocean_shader": Path("crates/s-renderer/src/ocean_surface.wgsl"),
    "flow_pipeline": Path("crates/s-renderer/src/flow_pipeline.rs"),
}

FLOW_WGSL = '''struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) depth: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.position = clip_position;
    out.depth = clamp(clip_position.z / max(clip_position.w, 0.0001), 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 距離に応じて少し透かし、FPS視点で遠近感を保つ。
    let alpha = mix(0.92, 0.24, input.depth);
    return vec4<f32>(0.08, 0.92, 1.0, alpha);
}
'''

PARTICLE_WGSL = '''struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) depth: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    out.position = clip_position;
    out.depth = clamp(clip_position.z / max(clip_position.w, 0.0001), 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 粒子は白。遠方は薄くし、シアンの流跡と瞬間位置を区別する。
    let alpha = mix(1.0, 0.55, input.depth);
    return vec4<f32>(1.0, 1.0, 1.0, alpha);
}
'''

OCEAN_WGSL = '''struct Camera {\n    view_proj: mat4x4<f32>,\n};\n\n@group(0) @binding(0)\nvar<uniform> camera: Camera;\n\nstruct VertexInput {\n    @location(0) position: vec3<f32>,\n    @location(1) height: f32,\n};\n\nstruct VertexOutput {\n    @builtin(position) position: vec4<f32>,\n};\n\n@vertex\nfn vs_main(input: VertexInput) -> VertexOutput {\n    var out: VertexOutput;\n    out.position = camera.view_proj * vec4<f32>(input.position, 1.0);\n    return out;\n}\n\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {\n    // 海面は青く半透明。実際のブレンドと深度設定は renderer.rs 側で行う。\n    return vec4<f32>(0.015, 0.30, 0.52, 0.30);\n}\n'''

def find_root() -> Path:
    here = Path.cwd().resolve()
    for candidate in [here, *here.parents]:
        if (candidate / FILES["renderer"]).is_file() and (candidate / FILES["flow_generator"]).is_file():
            return candidate
    raise RuntimeError("ワークスペースを見つけられません。crates/s-renderer/src/renderer.rs があるディレクトリで実行してください。")


def replace_particle_function(text: str) -> str:
    # Normalize both known source layouts. In the current layout, vertices() already
    # emits 3-axis crosses; keep that method trail-only so the separate white marker
    # pipeline can draw crosses without coloring them cyan or duplicating them.
    vertices_pattern = re.compile(
        r"(?m)^    ///[^\n]*\n(?:    ///[^\n]*\n)*    pub fn vertices\(&self\) -> Vec<FlowVertex> \{.*?^    \}\n",
        re.S,
    )
    vertices_replacement = '''    /// 各粒子の軌跡をGPU描画用の線分に変換する。
    pub fn vertices(&self) -> Vec<FlowVertex> {
        let mut vertices = Vec::new();

        for particle in &self.particles {
            for segment in particle.trail.windows(2) {
                vertices.push(FlowVertex { position: segment[0] });
                vertices.push(FlowVertex { position: segment[1] });
            }
        }

        vertices
    }
'''
    m = vertices_pattern.search(text)
    if not m:
        raise RuntimeError("flow_generator.rs の vertices 関数を安全に特定できません。ファイルは変更していません。")
    text = text[:m.start()] + vertices_replacement + text[m.end():]

    particle_pattern = re.compile(
        r"(?m)^    ///[^\n]*\n(?:    ///[^\n]*\n)*    pub fn particle_vertices\([^\n]*\) -> Vec<FlowVertex> \{.*?^    \}\n",
        re.S,
    )
    particle_replacement = '''    /// 現在の粒子位置を、3次元空間で視認しやすいマーカー頂点に変換する。
    /// 粒子の最終軌跡点を使うため、流線と同じ水深・座標を保つ。
    pub fn particle_vertices(&self) -> Vec<FlowVertex> {
        const MARKER_RADIUS: f32 = 450.0;
        let mut vertices = Vec::with_capacity(self.particles.len() * 6);

        for particle in &self.particles {
            let Some(&position) = particle.trail.last() else {
                continue;
            };
            let [x, y, z] = position;
            vertices.push(FlowVertex { position: [x - MARKER_RADIUS, y, z] });
            vertices.push(FlowVertex { position: [x + MARKER_RADIUS, y, z] });
            vertices.push(FlowVertex { position: [x, y - MARKER_RADIUS, z] });
            vertices.push(FlowVertex { position: [x, y + MARKER_RADIUS, z] });
            vertices.push(FlowVertex { position: [x, y, z - MARKER_RADIUS] });
            vertices.push(FlowVertex { position: [x, y, z + MARKER_RADIUS] });
        }

        vertices
    }
'''
    pm = particle_pattern.search(text)
    if pm:
        text = text[:pm.start()] + particle_replacement + text[pm.end():]
    else:
        count_pattern = re.compile(r"(?=    pub fn particle_count\(&self\) -> usize \{)")
        cm = count_pattern.search(text)
        if not cm:
            raise RuntimeError("flow_generator.rs に particle_count 関数がなく、particle_vertices の挿入位置を特定できません。ファイルは変更していません。")
        text = text[:cm.start()] + particle_replacement + "\n" + text[cm.start():]
    return text


def update_renderer(text: str) -> str:
    # Work only on the particle pipeline block, never replace the whole renderer file.
    m = re.search(r"^\s*let dot_pipeline = create_auxiliary_pipeline\(.*?^\s*\);", text, re.M | re.S)
    if not m:
        # Already renamed in a later local version.
        m = re.search(r"^\s*let (?:flow_dot_pipeline|particle_pipeline) = create_auxiliary_pipeline\(.*?^\s*\);", text, re.M | re.S)
    if not m:
        raise RuntimeError("renderer.rs 内の粒子描画パイプラインを特定できません。renderer.rs は変更していません。")

    block = m.group(0)
    if "particle_shader.module()" not in block:
        block = block.replace("flow_shader.module()", "particle_shader.module()")
    block = block.replace("wgpu::PrimitiveTopology::PointList", "wgpu::PrimitiveTopology::LineList")

    # alpha_blending is the first boolean argument after topology. If the helper
    # has a second depth-write argument, preserve it (it must remain false).
    lines = block.splitlines()
    seen_topology = False
    for i, line in enumerate(lines):
        if "wgpu::PrimitiveTopology::LineList" in line:
            seen_topology = True
            continue
        if seen_topology and re.match(r"^\s*(true|false),\s*$", line):
            if line.strip() == "false,":
                indent = line[:len(line) - len(line.lstrip())]
                lines[i] = indent + "true,"
            break
    block = "\n".join(lines)

    if "particle_shader.module()" not in block:
        raise RuntimeError("粒子パイプラインを particle.wgsl に切り替えられません。renderer.rs は変更していません。")

    text = text[:m.start()] + block + text[m.end():]
    if 'include_str!("particle.wgsl")' not in text:
        declaration = '''        let particle_shader = Shader::from_wgsl(
            context.device(),
            include_str!("particle.wgsl"),
            "particle_marker_shader",
        )?;

'''
        # Insert the shader declaration immediately before the dot-pipeline declaration.
        m2 = re.search(r"^\s*let (?:dot_pipeline|flow_dot_pipeline|particle_pipeline) = create_auxiliary_pipeline\(", text, re.M)
        if not m2:
            raise RuntimeError("粒子シェーダーの挿入位置を特定できません。renderer.rs は変更していません。")
        text = text[:m2.start()] + declaration + text[m2.start():]
    return text


def main() -> int:
    root = find_root()
    paths = {key: root / rel for key, rel in FILES.items()}
    missing = [str(p) for p in paths.values() if not p.is_file()]
    if missing:
        print("必要なファイルが見つかりません。変更は行いません:", *missing, sep="\n- ", file=sys.stderr)
        return 2

    original = {key: path.read_text(encoding="utf-8") for key, path in paths.items()}
    updated = dict(original)

    try:
        fg = updated["flow_generator"]
        fg = replace_particle_function(fg)
        # Flow geometry must remain at the sampled water depth; no constant lift to the sea surface.
        fg = fg.replace("                world_start[1] += 100.0;\n", "")
        fg = fg.replace("                world_end[1] += 100.0;\n", "")
        fg = fg.replace("            world_start[1] += 100.0;\n", "")
        fg = fg.replace("            world_end[1] += 100.0;\n", "")
        fg = fg.replace("        let dt = dt_seconds.clamp(0.0, 0.05) * speed_multiplier.max(0.0) * 100.0;",
                        "        // speed_multiplier はシミュレーション時間の倍率。流速値そのものは変更しない。\n        let dt = dt_seconds.clamp(0.0, 0.05) * speed_multiplier.max(0.0);")
        updated["flow_generator"] = fg

        ex = updated["example"]
        ex = ex.replace("velocity.w_mut().fill(0.05);", "// w が未提供のため、鉛直流を捏造せずゼロで初期化する。\n    velocity.w_mut().fill(0.0);")
        ex = ex.replace("GpuFlowMesh::with_capacity(renderer.device(), seeds.len());",
                        "GpuFlowMesh::with_capacity(renderer.device(), seeds.len() * 6);")
        ex = ex.replace("particles.particle_vertices(&converter)", "particles.particle_vertices()")
        updated["example"] = ex

        updated["renderer"] = update_renderer(updated["renderer"])
        fp = updated["flow_pipeline"]
        fp = fp.replace("depth_compare: wgpu::CompareFunction::Less,", "depth_compare: wgpu::CompareFunction::LessEqual,")
        updated["flow_pipeline"] = fp

        updated["flow_shader"] = FLOW_WGSL
        updated["particle_shader"] = PARTICLE_WGSL
        updated["ocean_shader"] = OCEAN_WGSL

        # Final guardrails against a partial/incorrect update.
        if "* 100.0;" in updated["flow_generator"] and "let dt = dt_seconds" in updated["flow_generator"]:
            raise RuntimeError("粒子時間倍率の旧式 *100 が残っています。変更は行いません。")
        if "velocity.w_mut().fill(0.05);" in updated["example"]:
            raise RuntimeError("未提供の鉛直流を設定するコードが残っています。変更は行いません。")
        if "wgpu::PrimitiveTopology::PointList" in updated["renderer"]:
            # Other point pipelines may be valid; only inspect the particle pipeline block.
            pm = re.search(r"let (?:dot_pipeline|flow_dot_pipeline|particle_pipeline) = create_auxiliary_pipeline\(.*?\n\s*\);", updated["renderer"], re.S)
            if pm and "wgpu::PrimitiveTopology::PointList" in pm.group(0):
                raise RuntimeError("粒子パイプラインが PointList のままです。変更は行いません。")
    except Exception as exc:
        print(f"更新を中止しました（ファイル未変更）: {exc}", file=sys.stderr)
        return 3

    changed = [key for key in paths if updated[key] != original[key]]
    if not changed:
        print("SOS海流表示の更新はすでに適用済みです。変更はありません。")
        return 0

    stamp = _dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    backups = []
    for key in changed:
        path = paths[key]
        backup = path.with_name(path.name + f".sos-backup-{stamp}")
        backup.write_text(original[key], encoding="utf-8")
        backups.append(backup)
    for key in changed:
        paths[key].write_text(updated[key], encoding="utf-8")

    print("SOS海流表示の更新を適用しました。")
    print("変更ファイル:")
    for key in changed:
        print(f"  - {paths[key].relative_to(root)}")
    print("バックアップ:")
    for backup in backups:
        print(f"  - {backup.relative_to(root)}")
    print("次に実行: cargo fmt --all -- --check && cargo check -p s-renderer --example terrain")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
