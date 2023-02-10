use fitsio::{FitsFile};
use fitsio::images::{ImageType, ImageDescription};
use fitsio::hdu::HduInfo;
use ndarray::{ArrayD, Ix4, Ix3};
use std::env;

fn rotate_fits_cube_axes_4d(fits_cube: ArrayD<f32>) -> ArrayD<f32> {
    let fits_cube_4d = fits_cube.into_dimensionality::<Ix4>().unwrap();
    let rotated_fits_cube = fits_cube_4d.permuted_axes((2, 0, 1, 3));
    rotated_fits_cube.into_dimensionality().unwrap()
}

fn rotate_fits_cube_axes_3d(fits_cube: ArrayD<f32>) -> ArrayD<f32> {
    let fits_cube_3d = fits_cube.into_dimensionality::<Ix3>().unwrap();
    let rotated_fits_cube = fits_cube_3d.permuted_axes((2, 0, 1));
    rotated_fits_cube.into_dimensionality().unwrap()
}

fn read_fits_cube(filename: &str) -> ArrayD<f32> {
    let mut f = FitsFile::open(filename).unwrap();
    let hdu = f.primary_hdu().unwrap();
    if let HduInfo::ImageInfo { shape, .. } = &hdu.info {
        println!("Image is {}-dimensional", shape.len());
        println!("Found image with shape {:?}", shape);
    }
    let data = hdu.read_image(&mut f).unwrap();
    data
}

fn write_fits_cube(filename: &str, fits_cube: ArrayD<f32>) {
    let description = ImageDescription {
        data_type: ImageType::Double,
        dimensions: &fits_cube.shape().to_vec(),
    };
    let mut fptr = FitsFile::create(filename)
    .with_custom_primary(&description)
    .open().unwrap();
    let hdu = fptr.create_image("EXTNAME".to_string(), &description).unwrap();
    hdu.write_image(&mut fptr, &fits_cube.into_raw_vec()).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <filename>", args[0]);
        return;
    }
    let filename = &args[1];
    let fits_cube = read_fits_cube(filename);
    println!("Original FITS cube shape: {:?}", fits_cube.shape());
    // Check if the FITS cube is 3D or 4D
    let rotated_fits_cube = match fits_cube.ndim() {
        3 => rotate_fits_cube_axes_3d(fits_cube.clone()),
        4 => rotate_fits_cube_axes_4d(fits_cube.clone()),
        _ => panic!("FITS cube must be 3D or 4D"),
    };
    println!("Original FITS cube shape: {:?}", fits_cube.shape());
    println!("Rotated FITS cube shape: {:?}", rotated_fits_cube.shape());
    let out_filename = filename.replace(".fits", ".rot.fits");
    write_fits_cube(&out_filename, rotated_fits_cube);
    print!("Wrote rotated FITS cube to {}", out_filename);
    print!("Done!")
}