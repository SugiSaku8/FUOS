# Streamline behavior when current speed reaches zero

`FlowParticleSystem` now clears a particle's trail when the sampled 3D speed
`hypot(u, v, w)` is at or below `1.0e-4 m/s` (0.1 mm/s). This small epsilon
handles floating-point residue around a physically stationary cell. The particle
remains at its current location rather than being reset to its seed; if flow resumes,
a new trail is generated from that location.

The existing rolling trail length remains bounded by `particle_trail_points` to
avoid unbounded memory and rendering cost. Thus moving particles still show a finite
trail, while a zero-speed particle's entire visible trail and marker disappear.
