import xarray as xr

from config import (
    LON_MIN,
    LON_MAX,
    LAT_MIN,
    LAT_MAX,
)


HYCOM_URL = (
    "https://ncss.hycom.org/thredds/dodsC/"
    "GLBy0.08/expt_93.0"
)


VARIABLES = [
    "water_temp",
    "salinity",
    "water_u",
    "water_v",
]


def download():

    print("Opening HYCOM...")

    ds = xr.open_dataset(
        HYCOM_URL,
        decode_times=False,
        drop_variables=[
            "tau",
            "surf_el",
            "water_temp_bottom",
            "salinity_bottom",
            "water_u_bottom",
            "water_v_bottom",
        ],
    )


    print("Selecting Suruga Bay...")


    ds = ds[
        VARIABLES
    ]


    ds = ds.sel(
        lon=slice(
            LON_MIN,
            LON_MAX
        ),
        lat=slice(
            LAT_MIN,
            LAT_MAX
        ),
        depth=slice(
            0,
            500
        )
    )


    #
    # 最新ではなく最初の解析時刻
    #

    ds = ds.isel(
        time=0
    )


    #
    # 通信chunk
    #

    ds = ds.chunk(
        {
            "depth": 10,
            "lat": 16,
            "lon": 16,
        }
    )


    print(ds)


    print("Saving...")

    ds.to_netcdf(
        "hycom_suruga_raw.nc",
        engine="netcdf4",
        compute=True,
    )


    print("Done.")



if __name__ == "__main__":
    download()
