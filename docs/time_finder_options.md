# Time Finder OPTIONS

The auxiliary Time Finder window now contains an `OPTIONS` control.

- Click `OPTIONS` in the upper-right to open/close its panel.
- Click the checkbox at the left of the panel to toggle temperature-anomaly / vertical-flow coloring. When disabled, trajectories return to a neutral cyan color.
- Click or drag the horizontal slider to change the number of displayed flow particles from 5% to 100% of available seed points. Seeds are sampled evenly across the existing seed list to avoid selecting only its beginning.
- The timeline remains in the lower band; clicking it seeks to a retained simulation snapshot.

The particle-count option changes visualization density only. It does not change the numerical ocean simulation or add physical terms to the solver. The temperature/upwelling coloring is diagnostic visualization and is not itself evidence that local upwelling is physically predicted accurately.

Validation note: source and archive integrity can be checked here, but Rust compilation and the macOS Metal runtime need to be verified on the target Mac because this environment has no Cargo/rustc toolchain.
