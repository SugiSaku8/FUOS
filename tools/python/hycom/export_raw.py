import xarray as xr
import numpy as np
import json
from pathlib import Path


INPUT = "suruga_boundary.nc"

OUTPUT = Path("suruga_raw")


def save_f32(path, array):

    array.astype(
        np.float32
    ).tofile(path)



def export():

    OUTPUT.mkdir(
        exist_ok=True
    )


    print("Opening dataset...")


    ds = xr.open_dataset(
        INPUT
    )


    print(ds)


    #
    # variables
    #

    temperature = (
        ds["temperature"]
        .values
    )

    salinity = (
        ds["salinity"]
        .values
    )

    u = (
        ds["u_velocity"]
        .values
    )

    v = (
        ds["v_velocity"]
        .values
    )


    #
    # NaN除去
    #

    temperature = np.nan_to_num(
        temperature,
        nan=0.0
    )

    salinity = np.nan_to_num(
        salinity,
        nan=0.0
    )

    u = np.nan_to_num(
        u,
        nan=0.0
    )

    v = np.nan_to_num(
        v,
        nan=0.0
    )


    #
    # save binary
    #

    save_f32(
        OUTPUT/"temperature.bin",
        temperature
    )


    save_f32(
        OUTPUT/"salinity.bin",
        salinity
    )


    save_f32(
        OUTPUT/"u.bin",
        u
    )


    save_f32(
        OUTPUT/"v.bin",
        v
    )


    #
    # grid metadata
    #

    depth = (
        ds.depth.values
        .tolist()
    )

    lat = (
        ds.lat.values
        .tolist()
    )

    lon = (
        ds.lon.values
        .tolist()
    )


    grid = {

        "time":1,

        "depth":len(depth),

        "height":len(lat),

        "width":len(lon)

    }


    with open(
        OUTPUT/"grid.json",
        "w"
    ) as f:

        json.dump(
            grid,
            f,
            indent=4
        )


    coordinates = {

        "depth":depth,

        "latitude":lat,

        "longitude":lon

    }


    with open(
        OUTPUT/"coordinates.json",
        "w"
    ) as f:

        json.dump(
            coordinates,
            f,
            indent=4
        )


    print(
        "Export complete"
    )


if __name__ == "__main__":

    export()
