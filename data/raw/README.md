# Public NetCDF input files

Place the downloaded public datasets here, or change `input.netcdf.terrain_file` and `input.netcdf.ocean_file` in `apps/simulator/config/simulator.json`.

Default filenames expected by the config:

- `gebco_suruga.nc`: GEBCO gridded bathymetry NetCDF
- `hycom_suruga.nc`: HYCOM (or a compatible ocean product) NetCDF containing u, v, temperature, salinity and depth coordinates

The files are intentionally not committed because public gridded ocean datasets can be large and are distributed separately by their providers.
