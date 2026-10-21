use anyhow::{Context, Result, ensure};
use s_field::{Coordinates, Field};
use s_renderer::{
    GeoBounds, GeoConverter, GpuFlowMesh, HeightMap, Renderer, TerrainMesh, TerrainVertex,
    camera_controller::CameraController,
    contour::generate_bathymetry_contours_with_scale,
    flow_generator::FlowParticleSystem,
    terrain::OceanSurfaceMesh,
};
use std::{collections::VecDeque, sync::Arc, time::Instant};
use winit::{dpi::PhysicalSize, event::{ElementState, Event, MouseButton, WindowEvent}, event_loop::EventLoop, window::WindowAttributes};

use crate::{config::Config, dataset::TerrainData, engine::SimulationEngine, time_finder::TimeFinder};

pub async fn run(config: Config, mut engine: SimulationEngine) -> Result<()> {
    println!("[5/7] リアルタイム表示システムを起動します");
    let terrain = engine.terrain().clone();
    let bounds = bounds_of(&terrain);
    let heightmap = HeightMap::new(terrain.width, terrain.height, terrain.bathymetry.clone());

    let event_loop = EventLoop::new()?;
    let window = Arc::new(event_loop.create_window(
        WindowAttributes::default().with_title("Suruga Ocean Simulator — Live Simulation")
    )?);
    let time_window = Arc::new(event_loop.create_window(
        WindowAttributes::default()
            .with_title("SOS 時間ファインダー")
            .with_inner_size(PhysicalSize::new(820, 250)),
    )?);
    let mut time_finder = pollster::block_on(TimeFinder::new(time_window.clone()))?;
    let main_window_id = window.id();
    let time_window_id = time_window.id();
    let mut time_cursor_x = 0.0_f64;
    let mut time_cursor_y = 0.0_f64;
    let mut options_open = false;
    let mut show_temperature = true;
    let mut flow_fraction = 1.0_f32;
    let mut flow_slider_dragging = false;
    let mut history: VecDeque<(u64, f64, s_field::OceanField)> = VecDeque::new();

    let terrain_mesh = TerrainMesh::from_heightmap_with_scale(
        &heightmap, bounds, config.grid.terrain_vertical_scale, config.grid.world_scale,
    );
    let ocean_mesh = OceanSurfaceMesh::new_with_scale(&bounds, &heightmap, config.grid.world_scale);
    println!("Terrain mesh: {} vertices / {} indices", terrain_mesh.vertex_count(), terrain_mesh.index_count());

    let mut renderer = Renderer::new(
        window.clone(),
        include_str!("../../../crates/s-renderer/src/terrain.wgsl"),
        &[TerrainVertex::layout()],
    ).await?;
    let gpu_terrain = terrain_mesh.upload(renderer.device());
    let gpu_ocean = ocean_mesh.upload(renderer.device());

    let contour_vertices = if config.render.show_bathymetry_contours {
        generate_bathymetry_contours_with_scale(
            &heightmap, bounds, config.grid.terrain_vertical_scale,
            config.render.contour_interval_m, config.grid.world_scale,
        )
    } else { Vec::new() };
    let gpu_contours = GpuFlowMesh::from_flow_vertices(renderer.device(), &contour_vertices);
    println!("Bathymetry contours: {} vertices", contour_vertices.len());

    let converter = GeoConverter::new(bounds, config.grid.world_scale);
    let coordinates = engine.field().velocity().coordinates()
        .context("初期海流場に座標がありません")?.clone();
    let seeds = generate_seeds(
        &coordinates, &bounds, &terrain, config.render.seed_stride_x, config.render.seed_stride_y,
    );
    ensure!(!seeds.is_empty(), "海底地形と海流格子が重なる領域に粒子の開始点がありません");
    let seed_points = seeds.clone();
    let mut active_seeds = seed_points.clone();
    let mut last_flow_count = active_seeds.len();
    let mut particles = FlowParticleSystem::new_with_vertical_scale(&active_seeds, config.render.particle_trail_points, config.grid.terrain_vertical_scale);
    history.push_back((engine.step(), engine.time_seconds(), engine.snapshot()));
    let snapshot_interval = config.simulation.steps_per_frame.max(1) as u64 * 10;
    let max_vertices = seeds.len() * ((config.render.particle_trail_points - 1) * 2 + 6);
    let mut gpu_flow = GpuFlowMesh::with_capacity(renderer.device(), max_vertices);
    let gpu_particle_dots = GpuFlowMesh::with_capacity(renderer.device(), seeds.len());
    let mut camera_controller = CameraController::new();
    println!("Flow seeds: {}, depth levels: {}", seeds.len(), coordinates.depth_levels());
    println!("[6/7] シミュレーションと描画を同じ実行ループで開始します");

    let mut last_frame = Instant::now();
    let mut report_elapsed = 0.0_f32;
    let mut report_frames = 0_u64;
    let mut title_elapsed = 0.0_f32;

    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { window_id, event: WindowEvent::CloseRequested } if window_id == main_window_id => {
                if let Err(error) = engine.finish_early() { eprintln!("Final save error: {error:#}"); }
                target.exit();
            }
            Event::WindowEvent { window_id, event: WindowEvent::CloseRequested } if window_id == time_window_id => {
                // Closing the finder only hides the auxiliary view; simulation continues.
                time_window.set_visible(false);
            }
            Event::WindowEvent { window_id, event: WindowEvent::Resized(size) } if window_id == main_window_id => renderer.resize(size),
            Event::WindowEvent { window_id, event: WindowEvent::Resized(size) } if window_id == time_window_id => time_finder.resize(size),
            Event::AboutToWait => { window.request_redraw(); if time_window.is_visible().unwrap_or(false) { time_window.request_redraw(); } },
            Event::WindowEvent { window_id, event: WindowEvent::CursorMoved { position, .. } } if window_id == time_window_id => {
                time_cursor_x = position.x;
                time_cursor_y = position.y;
                if options_open && flow_slider_dragging {
                    let width = time_window.inner_size().width.max(1) as f64;
                    let nx = (position.x / width * 2.0 - 1.0) as f32;
                    flow_fraction = (((nx + 0.60) / 1.42).clamp(0.0, 1.0) * 0.95 + 0.05).clamp(0.05, 1.0);
                    let desired = ((seed_points.len() as f32 * flow_fraction).round() as usize).clamp(1, seed_points.len().max(1));
                    active_seeds = select_seed_subset(&seed_points, desired);
                    last_flow_count = active_seeds.len();
                    particles = FlowParticleSystem::new_with_vertical_scale(&active_seeds, config.render.particle_trail_points, config.grid.terrain_vertical_scale);
                }
            }
            Event::WindowEvent { window_id, event: WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } } if window_id == time_window_id => {
                flow_slider_dragging = false;
            }
            Event::WindowEvent { window_id, event: WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } } if window_id == time_window_id => {
                let size = time_window.inner_size();
                let width = size.width.max(1) as f64;
                let height = size.height.max(1) as f64;
                let nx = (time_cursor_x / width * 2.0 - 1.0) as f32;
                let ny = (1.0 - time_cursor_y / height * 2.0) as f32;
                if nx >= 0.50 && ny >= 0.56 {
                    options_open = !options_open;
                } else if options_open && (-0.94..=-0.78).contains(&nx) && (0.24..=0.48).contains(&ny) {
                    show_temperature = !show_temperature;
                    println!("OPTIONS: 温度可視化 {}", if show_temperature { "ON" } else { "OFF" });
                } else if options_open && (-0.64..=0.86).contains(&nx) && (0.24..=0.48).contains(&ny) {
                    flow_slider_dragging = true;
                    flow_fraction = (((nx + 0.60) / 1.42).clamp(0.0, 1.0) * 0.95 + 0.05).clamp(0.05, 1.0);
                    let desired = ((seed_points.len() as f32 * flow_fraction).round() as usize).clamp(1, seed_points.len().max(1));
                    active_seeds = select_seed_subset(&seed_points, desired);
                    last_flow_count = active_seeds.len();
                    particles = FlowParticleSystem::new_with_vertical_scale(&active_seeds, config.render.particle_trail_points, config.grid.terrain_vertical_scale);
                    println!("OPTIONS: 流線 {}本 / 全{}本", last_flow_count, seed_points.len());
                } else if ny >= -0.82 && ny <= -0.38 && nx >= -0.96 && nx <= 0.96 {
                    if history.len() > 1 {
                        let fraction = (time_cursor_x / width).clamp(0.0, 1.0);
                        let index = ((history.len() - 1) as f64 * fraction).round() as usize;
                        if let Some((step, time, field)) = history.get(index).map(|(s, t, f)| (*s, *t, (*f).clone())) {
                            history.truncate(index + 1);
                            engine.restore_snapshot(field, step, time);
                            particles = FlowParticleSystem::new_with_vertical_scale(&active_seeds, config.render.particle_trail_points, config.grid.terrain_vertical_scale);
                            println!("Time Finder: seek to step={} t={:.3}s", step, time);
                        }
                    }
                }
            }
            Event::WindowEvent { window_id, event: WindowEvent::RedrawRequested } if window_id == time_window_id => {
                let max_time = config.simulation.duration_seconds.max(1.0);
                let history_start = history.front().map(|(_, t, _)| *t).unwrap_or(0.0);
                let history_end = history.back().map(|(_, t, _)| *t).unwrap_or(max_time);
                let span = (history_end - history_start).max(1.0e-9);
                time_finder.render(
                    ((engine.time_seconds() - history_start) / span) as f32,
                    history.len(), engine.time_seconds(), max_time,
                    options_open, show_temperature, flow_fraction,
                );
            }
            Event::WindowEvent { window_id, event: WindowEvent::RedrawRequested } if window_id == main_window_id => {
                let now = Instant::now();
                let wall_dt = now.duration_since(last_frame).as_secs_f32();
                last_frame = now;

                if !engine.is_finished() {
                    if let Err(error) = engine.advance(config.simulation.steps_per_frame) {
                        eprintln!("Simulation error: {error:#}");
                        if let Err(save_error) = engine.finish_early() {
                            eprintln!("Could not save partial state: {save_error:#}");
                        }
                    }
                }

                if engine.step() % snapshot_interval == 0 {
                    let should_push = history.back().map(|(step, _, _)| *step != engine.step()).unwrap_or(true);
                    if should_push {
                        history.push_back((engine.step(), engine.time_seconds(), engine.snapshot()));
                        while history.len() > 180 { history.pop_front(); }
                    }
                }
                if show_temperature {
                    particles.update_with_temperature(
                        engine.field().velocity(), engine.field().temperature().field().as_slice(), &converter, wall_dt,
                        config.render.particle_speed_multiplier, engine.time_seconds(),
                    );
                } else {
                    particles.update(
                        engine.field().velocity(), &converter, wall_dt,
                        config.render.particle_speed_multiplier, engine.time_seconds(),
                    );
                }
                let flow_vertices = particles.vertices();
                if !gpu_flow.update_vertices(renderer.queue(), &flow_vertices) {
                    eprintln!("Flow vertex capacity exceeded: {} > {}", flow_vertices.len(), max_vertices);
                }
                camera_controller.update_with_dt(renderer.camera_mut(), wall_dt);
                // Prevent the FPS camera from crossing the local bathymetry surface.
                let camera = renderer.camera_mut();
                let world_scale = config.grid.world_scale.max(1.0);
                // Keep the camera inside the rendered terrain domain, with a small
                // margin so the view cannot drift into undefined bathymetry.
                let margin = (world_scale * 0.002).clamp(50.0, 500.0);
                camera.position.x = camera.position.x.clamp(margin, (world_scale - margin).max(margin));
                camera.position.z = camera.position.z.clamp(margin, (world_scale - margin).max(margin));
                let tx = ((camera.position.x / world_scale).clamp(0.0, 1.0) * (terrain.width.saturating_sub(1)) as f32).round() as usize;
                let ty = ((camera.position.z / world_scale).clamp(0.0, 1.0) * (terrain.height.saturating_sub(1)) as f32).round() as usize;
                let seabed_y = terrain.bathymetry[ty.min(terrain.height-1) * terrain.width + tx.min(terrain.width-1)] * config.grid.terrain_vertical_scale;
                camera.position.y = camera.position.y.max(seabed_y + 250.0);
                report_elapsed += wall_dt;
                report_frames += 1;
                title_elapsed += wall_dt;
                if title_elapsed >= 0.5 {
                    let state = if engine.is_finished() { "完了" } else { "計算中" };
                    window.set_title(&format!(
                        "Suruga Ocean Simulator | {} | step {} | t={:.2}/{:.2}s",
                        state, engine.step(), engine.time_seconds(), config.simulation.duration_seconds
                    ));
                    title_elapsed = 0.0;
                }
                if report_elapsed >= 2.0 {
                    println!("FPS: {:.1} | step: {} | sim t: {:.2}s / {:.2}s | flow vertices: {}",
                        report_frames as f32 / report_elapsed, engine.step(), engine.time_seconds(),
                        config.simulation.duration_seconds, flow_vertices.len());
                    report_elapsed = 0.0;
                    report_frames = 0;
                }

                let ocean = if config.render.show_ocean_surface { Some(&gpu_ocean) } else { None };
                let contours = if config.render.show_bathymetry_contours { Some(&gpu_contours) } else { None };
                if let Err(error) = renderer.render_with_ocean_and_particles(
                    &gpu_terrain, ocean, contours, Some(&gpu_flow), Some(&gpu_particle_dots), None,
                ) {
                    eprintln!("Render error: {error:#}");
                }
            }
            Event::WindowEvent { window_id, event } if window_id == main_window_id => camera_controller.process(&event),
            _ => {}
        }
    })?;
    Ok(())
}

fn select_seed_subset(seeds: &[[f32; 3]], desired: usize) -> Vec<[f32; 3]> {
    if seeds.is_empty() || desired == 0 { return Vec::new(); }
    if desired >= seeds.len() { return seeds.to_vec(); }
    // Evenly distribute selected seeds across the full depth/area rather than
    // merely truncating the list and biasing the display toward shallow cells.
    (0..desired).map(|i| {
        let index = i.saturating_mul(seeds.len() - 1) / desired.saturating_sub(1).max(1);
        seeds[index]
    }).collect()
}

fn bounds_of(terrain: &TerrainData) -> GeoBounds {
    let lon_min = terrain.longitude.iter().copied().fold(f32::INFINITY, f32::min) as f64;
    let lon_max = terrain.longitude.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let lat_min = terrain.latitude.iter().copied().fold(f32::INFINITY, f32::min) as f64;
    let lat_max = terrain.latitude.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    GeoBounds::new(lon_min, lon_max, lat_min, lat_max)
}

fn generate_seeds(
    coordinates: &Coordinates,
    bounds: &GeoBounds,
    terrain: &TerrainData,
    stride_x: usize,
    stride_y: usize,
) -> Vec<[f32; 3]> {
    let mut seeds = Vec::new();
    for &depth in coordinates.depth() {
        for y in (0..coordinates.height()).step_by(stride_y) {
            for x in (0..coordinates.width()).step_by(stride_x) {
                let Some(lon) = coordinates.longitude_at(x) else { continue; };
                let Some(lat) = coordinates.latitude_at(y) else { continue; };
                if lon < bounds.lon_min || lon > bounds.lon_max || lat < bounds.lat_min || lat > bounds.lat_max { continue; }
                let tx = nearest_index(&terrain.longitude, lon as f32).min(terrain.width - 1);
                let ty = nearest_index(&terrain.latitude, lat as f32).min(terrain.height - 1);
                let elevation = terrain.bathymetry[ty * terrain.width + tx];
                if elevation < 0.0 && depth <= -elevation {
                    seeds.push([lon as f32, lat as f32, depth]);
                }
            }
        }
    }
    seeds
}

fn nearest_index(values: &[f32], value: f32) -> usize {
    values.iter().enumerate()
        .min_by(|(_, a), (_, b)| (**a - value).abs().total_cmp(&(**b - value).abs()))
        .map(|(index, _)| index).unwrap_or(0)
}
