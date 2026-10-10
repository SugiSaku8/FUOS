use s_field::OceanField;
use s_metal::{GpuOceanState, MetalContext, OceanSolver};

pub struct Simulation {
    pub field: OceanField,
    pub state: GpuOceanState,
    pub solver: OceanSolver,
    pub time: f32,
}

impl Simulation {
    pub fn new(context: &MetalContext, field: OceanField, solver: OceanSolver) -> Self {
        let state = GpuOceanState::from_ocean(context, &field);

        Self {
            field,
            state,
            solver,
            time: 0.0,
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.solver.step(&mut self.state, dt);

        self.time += dt;
    }
}
