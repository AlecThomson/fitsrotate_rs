#![crate_name = "fitsrotate_rs"]
#![allow(unused)]
use fitsio::images::{ImageDescription, ImageType};
#[doc(inline)]
use fitsio::FitsFile;
use ndarray::ArrayD;
use std::path::Path;
use clap::Parser;

/// Convert a FITS index to an array index
///
/// FITS indices are 1-based, while array indices are 0-based.
/// FITS indices are in the order x, y, z, ..., while array indices are in the order z, y, x, ...
///
/// # Arguments
///
/// * `fits_index` - The FITS index
/// * `naxis` - The number of axes in the FITS cube
///
/// # Returns
///
/// * `usize` - The array index
///
/// # Examples
///
/// ```
/// use fitsrotate_rs::fits_index_to_array_index;
/// let fits_index = 3;
/// let naxis = 3;
/// let array_index = fits_index_to_array_index(fits_index, naxis);
/// assert_eq!(array_index, 2);
/// ```
///
/// ```
/// use fitsrotate_rs::fits_index_to_array_index;
/// let fits_index = 1;
/// let naxis = 3;
/// let array_index = fits_index_to_array_index(fits_index, naxis);
/// assert_eq!(array_index, 0);
/// ```
///
/// ```
/// use fitsrotate_rs::fits_index_to_array_index;
/// let fits_index = 2;
/// let naxis = 3;
/// let array_index = fits_index_to_array_index(fits_index, naxis);
/// assert_eq!(array_index, 1);
/// ```
fn fits_index_to_array_index(fits_index: usize, naxis: usize) -> usize {
    let range = (0..(naxis as i32)).rev();
    let ret = Vec::from_iter(range)[fits_index - 1];
    println!("fits_index_to_array_index: {} -> {}", fits_index, ret);
    return ret as usize;
}

/// Rotate the axes of a FITS cube so that the frequency axis is the last axis
///
/// # Arguments
///
/// * `fits_cube` - The FITS cube
/// * `fits_file` - The FITS file
///
/// # Returns
///
/// * `ArrayD<f32>` - The rotated FITS cube
/// * `usize` - The index of the frequency axis
///
/// # Examples
///
/// ```
/// use fitsrotate_rs::rotate_fits_cube_axes;
/// use fitsrotate_rs::rotate_fits_cube_axes;
/// let fits_cube = ArrayD::zeros((3, 3, 3));
/// let mut fits_file = FitsFile::open(filename).unwrap();
/// let (rotated_fits_cube, freq_axis) = rotate_fits_cube_axes(fits_cube, &mut fits_file);
/// ```
fn rotate_fits_cube_axes(fits_cube: ArrayD<f32>, fits_file: &mut FitsFile) -> (ArrayD<f32>, usize) {
    let shape = fits_cube.shape();
    let axes: Vec<usize> = (0..shape.len()).collect();
    // Find the axis that corresponds to the frequency axis
    for fits_idx in 1..shape.len() + 1 {
        let card = "CTYPE".to_owned() + &fits_idx.to_string();
        let hdu = fits_file.hdu(0).unwrap();
        let head_val: String = hdu.read_key(fits_file, &card).unwrap();
        if head_val == "FREQ" {
            println!("Found frequency axis at index {}", fits_idx);
            let mut new_axes = axes.clone();
            let idx = fits_index_to_array_index(fits_idx, shape.len());
            new_axes.remove(idx);
            new_axes.push(idx);
            println!("New axes: {:?}", new_axes);
            let rotated_fits_cube = fits_cube.permuted_axes(new_axes);
            return (rotated_fits_cube.into_dimensionality().unwrap(), fits_idx);
        } else {
            println!("Found axis {} with CTYPE {}", fits_idx, head_val);
        }
    }
    panic!("Could not find frequency axis");
}

/// Read a FITS cube
///
/// # Arguments
///
/// * `filename` - The FITS file
///
/// # Returns
///
/// * `ArrayD<f32>` - The FITS cube
/// * `FitsFile` - The FITS file
///
/// # Examples
///
/// ```
/// use fitsrotate_rs::read_fits_cube;
/// let (fits_cube, fits_file) = read_fits_cube("test.fits");
/// ```
fn read_fits_cube(filename: &str) -> (ArrayD<f32>, FitsFile) {
    let mut fits_file = FitsFile::open(filename).unwrap();
    let hdu = fits_file.primary_hdu().unwrap();
    let data = hdu.read_image(&mut fits_file).unwrap();
    return (data, fits_file);
}

/// Write a FITS cube
///
/// # Arguments
///
/// * `filename` - The FITS file
/// * `fits_cube` - The FITS cube
/// * `old_spec_idx` - The index of the frequency axis in the original FITS cube
/// * `old_file` - The original FITS file
/// * `overwrite` - Overwrite the FITS file if it already exists
///
/// # Examples
///
/// ```
/// use fitsrotate_rs::write_fits_cube;
/// write_fits_cube("test.fits", fits_cube, old_spec_idx, old_file, true);
/// ```
fn write_fits_cube(
    filename: &str,
    fits_cube: ArrayD<f32>,
    old_spec_idx: usize,
    mut old_file: FitsFile,
    overwrite: bool,
) {
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
        dimensions: &fits_cube.shape(),
    };
    let mut fits_file = FitsFile::create(filename)
        .with_custom_primary(&description)
        .open()
        .unwrap();

    let hdu = fits_file.hdu(0).unwrap();
    hdu.copy_to(&mut old_file, &mut fits_file).unwrap();

    let new_spec_idx: usize = 1;
    let shape = fits_cube.shape();
    // Swap the CTYPE and CRVAL for the spectral axis

    for card_stub in ["CTYPE", "CRVAL", "CDELT", "CRPIX", "CUNIT"] {
        for fits_idx in 1..shape.len() + 1 {
            if fits_idx == old_spec_idx {
                let old_card = card_stub.to_owned() + &fits_idx.to_string();
                let new_card = card_stub.to_owned() + &new_spec_idx.to_string();
                let head_val: String = hdu.read_key(&mut old_file, &old_card).unwrap();
                hdu.write_key(&mut fits_file, &new_card, head_val).unwrap();
            } else if fits_idx == new_spec_idx {
                let old_card = card_stub.to_owned() + &fits_idx.to_string();
                let new_card = card_stub.to_owned() + &old_spec_idx.to_string();
                let head_val: String = hdu.read_key(&mut old_file, &old_card).unwrap();
                hdu.write_key(&mut fits_file, &new_card, head_val).unwrap();
            } else {
                continue;
            }
        }
    }
    hdu.write_image(&mut fits_file, &fits_cube.into_raw_vec())
        .unwrap();
}

/// Simple program rotating the axes of a FITS cube
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The FITS file
    filename: String,
    /// Overwrite the FITS file if it already exists
    #[arg(short='o', long="overwrite")]
    overwrite: bool,
}

fn main() {
    let args = Args::parse();

    let filename = args.filename;
    let (fits_cube, mut fits_file) = read_fits_cube(&filename);

    println!("Original FITS cube shape: {:?}", fits_cube.shape());
    let (rotated_fits_cube, old_spec_idx) =
        rotate_fits_cube_axes(fits_cube.clone(), &mut fits_file);
    println!("Original FITS cube shape: {:?}", fits_cube.shape());
    println!("Rotated FITS cube shape: {:?}", rotated_fits_cube.shape());
    let out_filename = filename.replace(".fits", ".rot.fits");
    write_fits_cube(
        &out_filename,
        rotated_fits_cube,
        old_spec_idx,
        fits_file,
        args.overwrite,
    );
    println!("Wrote rotated FITS cube to {}", out_filename);
    println!("Done!");
}
