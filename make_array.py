import dask.array as da
import numpy as np
from astropy.io import fits
from astropy.wcs import WCS


def main(
        num_dimensions: int=3,
):
    npix_x = 1024
    npix_y = 1024
    nchan = 288
    nstokes = 3

    if num_dimensions < 2 or num_dimensions > 4:
        raise ValueError("num_dimensions must be 2, 3, or 4")

    if num_dimensions == 2:
        large_array = da.zeros(
            (nchan, npix_y), 
            chunks=(1, npix_y), 
            dtype=np.float32
        )
    elif num_dimensions == 3:
        large_array = da.zeros(
            (nchan, npix_y, npix_x), 
            chunks=(1, npix_y, npix_x), 
            dtype=np.float32
        )
    elif num_dimensions == 4:
        large_array = da.zeros(
            (nchan, nstokes, npix_y, npix_x), 
            chunks=(1, 1, npix_y, npix_x), 
            dtype=np.float32
        )

    # Create a header
    header = fits.Header()
    header["NAXIS"] = num_dimensions

    dims = [nchan, nstokes, npix_y, npix_x]
    types = ["FREQ", "STOKES", "RA--SIN", "DEC--SIN"]
    vals = [1.4e9, 1, 0.0, 0.0]
    delts = [1e6, 1, -1/3600, 1/3600]
    pixs = [1, 1, npix_y//2+1, npix_x//2+1]
    units = ["Hz", "", "deg", "deg"]

    for i, fits_idx in enumerate(range(num_dimensions, 0, -1)):
        header[f"NAXIS{fits_idx}"] = dims[i]
        header[f"CTYPE{fits_idx}"] = types[i]
        header[f"CRVAL{fits_idx}"] = vals[i]
        header[f"CDELT{fits_idx}"] = delts[i]
        header[f"CRPIX{fits_idx}"] = pixs[i]
        header[f"CUNIT{fits_idx}"] = units[i]

    header["BUNIT"] = "Jy/beam"
    header["BMAJ"] = 10 / 3600
    header["BMIN"] = 10 / 3600
    header["BPA"] = 0.0
    header["EQUINOX"] = 2000.0
    header["RADESYS"] = "FK5"
    header["LONPOLE"] = 180.0
    header["LATPOLE"] = 0.0
    header["RESTFRQ"] = 1.4e9
    header["SPECSYS"] = "LSRK"

    # Write to file
    fits.writeto(f"large_{num_dimensions}_array.fits", large_array, header, overwrite=True)


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-n",
        "--num-dimensions",
        type=int,
        default=3,
        help="Number of dimensions for the array"
    )
    args = parser.parse_args()
    main(
        num_dimensions=args.num_dimensions,
    )

