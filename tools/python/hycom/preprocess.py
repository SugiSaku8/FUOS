import xarray as xr
import numpy as np


DEPTH_LEVELS = [
    0,
    10,
    20,
    50,
    100,
    200,
]


def preprocess():

    ds = xr.open_dataset(
        "hycom_suruga_raw.nc"
    )


    #
    # depth interpolation
    #

    ds = ds.interp(
        depth=DEPTH_LEVELS
    )


    #
    # SOS variable format
    #

    output = xr.Dataset(
        {

            "temperature":
                ds["water_temp"]
                .astype(
                    np.float32
                ),


            "salinity":
                ds["salinity"]
                .astype(
                    np.float32
                ),


            "u_velocity":
                ds["water_u"]
                .astype(
                    np.float32
                ),


            "v_velocity":
                ds["water_v"]
                .astype(
                    np.float32
                ),

        }
    )
    output = output.fillna(0.0)

    output.to_netcdf(
        "suruga_boundary.nc"
    )


    print(output)



if __name__ == "__main__":
    preprocess()
