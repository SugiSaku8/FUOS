import xarray as xr
import numpy as np
from pathlib import Path


INPUT = Path("suruga.nc")
OUTPUT = Path("bathymetry.bin")


def main():

    print("Opening:", INPUT)

    ds = xr.open_dataset(INPUT)

    print(ds)

    #
    # bathymetry variable detection
    #

    candidates = [
        "bathymetry",
        "depth",
        "elevation",
        "h",
        "z",
    ]

    variable = None

    for name in candidates:
        if name in ds:
            variable = name
            break

    if variable is None:
        raise RuntimeError(
            "Bathymetry variable not found"
        )


    print("Using variable:", variable)


    bathy = ds[variable].values


    print("Original shape:", bathy.shape)


    #
    # 2D化
    #

    while bathy.ndim > 2:
        bathy = bathy[0]


    bathy = np.asarray(
        bathy,
        dtype=np.float32
    )


    print(
        "Final shape:",
        bathy.shape
    )


    print(
        "min/max:",
        bathy.min(),
        bathy.max()
    )


    #
    # Rust用raw binary
    #

    bathy.tofile(
        OUTPUT
    )


    print(
        "written:",
        OUTPUT
    )


if __name__ == "__main__":
    main()
