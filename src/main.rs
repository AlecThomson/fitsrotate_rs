use fitsio::{FitsFile};
use fitsio::images::{ImageType, ImageDescription};
// use fitsio::hdu::{HduInfo, FitsHdu};
use ndarray::{ArrayD, Ix4, Ix3, Array};
use std::env;
use std::path::Path;

fn rotate_fits_cube_axes_4d(fits_cube: ArrayD<f32>) -> ArrayD<f32> {
    let fits_cube_4d = fits_cube.into_dimensionality::<Ix4>().unwrap();
    // Order of axes is (chan, stokes, y, x) - (0, 1, 2, 3)
    // Rotate to (stokes, y, x, chan) - (1, 2, 3, 0)
    let rotated_fits_cube = fits_cube_4d.permuted_axes((1, 2, 3, 0));
    return rotated_fits_cube.into_dimensionality().unwrap()
}

fn rotate_fits_cube_axes_3d(fits_cube: ArrayD<f32>) -> ArrayD<f32> {
    // Order of axes is (chan, y, x) - (0, 1, 2)
    // Rotate to (y, x, chan) - (1, 2, 0)
    let fits_cube_3d = fits_cube.into_dimensionality::<Ix3>().unwrap();
    let rotated_fits_cube = fits_cube_3d.permuted_axes((1, 2, 0));
    return rotated_fits_cube.into_dimensionality().unwrap()
}

fn fits_index_to_array_index(fits_index: usize, naxis: usize) -> usize {
    let range = (0..(naxis as i32)).rev();
    let ret = Vec::from_iter(range)[fits_index];
    return ret as usize;
}

fn rotate_fits_cube_axes(fits_cube: ArrayD<f32>, fits_file: &mut FitsFile) -> ArrayD<f32> {
    let shape = fits_cube.shape();
    let axes: Vec<usize> = (0..shape.len()).collect();
    // Find the axis that corresponds to the frequency axis
    for i in 0..shape.len() {
        let card = "CTYPE".to_owned() + &(i + 1).to_string();
        let hdu = fits_file.hdu(0).unwrap();
        let head_val: String = hdu.read_key(fits_file, &card).unwrap();
        if head_val == "FREQ" {
            println!("Found frequency axis at index {}", i);
            let mut new_axes = axes.clone();
            let idx = fits_index_to_array_index(i, shape.len());
            new_axes.remove(idx);
            new_axes.push(idx);
            println!("New axes: {:?}", new_axes);
            let rotated_fits_cube = fits_cube.permuted_axes(new_axes);
            return rotated_fits_cube.into_dimensionality().unwrap();
        }
        else {
            println!("Found axis {} with CTYPE {}", i, head_val);
        }
    }
    panic!("Could not find frequency axis");
}

fn read_fits_cube(filename: &str) -> (ArrayD<f32>, FitsFile) {
    let mut fits_file = FitsFile::open(filename).unwrap();
    let hdu = fits_file.primary_hdu().unwrap();
    // if let HduInfo::ImageInfo { shape, .. } = &hdu.info {
    //     println!("Image is {}-dimensional", shape.len());
    //     println!("Found image with shape {:?}", shape);
    // }
    let data = hdu.read_image(&mut fits_file).unwrap();
    return (data, fits_file);
}

fn write_fits_cube(filename: &str, fits_cube: ArrayD<f32>, overwrite: bool) {
    // Check if file exists
    if Path::new(filename).exists() {
        if overwrite {
            std::fs::remove_file(filename).unwrap();
            println!("File {} already exists, overwriting", filename)
        } else {
            panic!("File {} already exists", filename);
        }
    }

    let description = ImageDescription {
        data_type: ImageType::Double,
        dimensions: &fits_cube.shape().to_vec(),
    };
    let mut fits_file = FitsFile::create(filename)
    .with_custom_primary(&description)
    .open().unwrap();
    // let hdu = fits_file.create_image("EXTNAME".to_string(), &description).unwrap();
    let hdu = fits_file.hdu(0).unwrap();
    hdu.write_image(&mut fits_file, &fits_cube.into_raw_vec()).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <filename> ", args[0]);
        return;
    }
    let filename = &args[1];
    let reader = read_fits_cube(filename);
    let fits_cube = reader.0;
    let mut fits_file = reader.1;
    println!("Original FITS cube shape: {:?}", fits_cube.shape());
    let rotated_fits_cube = rotate_fits_cube_axes(fits_cube.clone(), &mut fits_file);
    // // Check if the FITS cube is 3D or 4D
    // let rotated_fits_cube = match fits_cube.ndim() {
    //     3 => rotate_fits_cube_axes_3d(fits_cube.clone()),
    //     4 => rotate_fits_cube_axes_4d(fits_cube.clone()),
    //     _ => panic!("FITS cube must be 3D or 4D"),
    // };
    println!("Original FITS cube shape: {:?}", fits_cube.shape());
    println!("Rotated FITS cube shape: {:?}", rotated_fits_cube.shape());
    let out_filename = filename.replace(".fits", ".rot.fits");
    write_fits_cube(&out_filename, rotated_fits_cube, true);
    println!("Wrote rotated FITS cube to {}", out_filename);
    println!("Done!");
}