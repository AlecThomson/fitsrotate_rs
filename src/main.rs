#![crate_name = "fitsrotate_rs"]
#![allow(unused)]
use fitsio::images::{ImageDescription, ImageType};
#[doc(inline)]
use fitsio::FitsFile;
use fitsio::errors::Error;
use ndarray::ArrayD;
use std::path::Path;
use clap::{builder::Str, Parser};

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
    ret as usize
}

/// Rotate the axes of a FITS cube array given some new ordering
///
/// # Arguments
///
/// * `fits_cube` - The FITS cube
/// * `fits_file` - The FITS file
/// * `mode` - The new ordering of the axes
///
/// # Returns
///
/// * `ArrayD<f32>` - The rotated FITS cube
///
/// # Examples
///
/// ```
/// use fitsrotate_rs::rotate_fits_cube_axes;
/// use fitsrotate_rs::rotate_fits_cube_axes;
/// let fits_cube = ArrayD::zeros((3, 3, 3));
/// let mut fits_file = FitsFile::open(filename).unwrap();
/// let mode = [3, 2, 1];
/// let (rotated_fits_cube, freq_axis) = rotate_fits_cube_axes(fits_cube, &mut fits_file, &mode);
/// ```
fn rotate_fits_cube_axes(fits_cube: ArrayD<f32>, fits_file: &mut FitsFile, mode: &[usize]) -> ArrayD<f32> {
    let shape = fits_cube.shape();
    let old_axes: Vec<usize> = (0..shape.len()).collect();
    let old_mode:Vec<usize> = (1..shape.len()+1).collect();
    let new_axes: Vec<usize> = mode.iter().map(|x| x - 1).collect();

    // Just shift the data here
    let rot_cube = fits_cube.permuted_axes(new_axes);
    println!("New axes: {:?}", rot_cube.shape());
    rot_cube
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
    (data, fits_file)
}


/// Check if a file exists
/// 
/// # Arguments
/// 
/// * `filename` - The file to check
/// * `overwrite` - Overwrite the file if it already exists
/// 
/// # Returns
/// 
/// * `Result<bool, Error>` - True if the file exists
/// 
fn check_file_exists(filename: &str, overwrite: bool) -> Result<bool, Error> {
    if ! overwrite && Path::new(filename).exists() {
        return Err(Error::ExistingFile(filename.to_string()));
    }
    Ok(true)
}

/// Write a FITS cube
///
/// # Arguments
///
/// * `filename` - The FITS file
/// * `fits_cube` - The FITS cube
/// * `mode` - The new ordering of the axes
/// * `old_file` - The original FITS file
/// * `overwrite` - Overwrite the FITS file if it already exists
///
/// # Examples
///
/// ```
/// use fitsrotate_rs::write_fits_cube;
/// write_fits_cube("test.fits", fits_cube, mode, old_file, true);
/// ```
fn write_fits_cube(
    filename: &str,
    fits_cube: ArrayD<f32>,
    mode: &[usize],
    mut old_file: FitsFile,
    overwrite: bool,
) -> Result<(), Error>{
    // Check if file exists
    if Path::new(filename).exists() {
        if overwrite {
            std::fs::remove_file(filename)?;
            println!("File {} already exists, overwriting", filename);
        } else {
            return Err(Error::ExistingFile(filename.to_string()));
        }
    };

    let description = ImageDescription {
        data_type: ImageType::Double,
        dimensions: fits_cube.shape(),
    };
    let mut fits_file = FitsFile::create(filename)
        .with_custom_primary(&description)
        .open()?;

    let hdu = fits_file.hdu(0)?;
    // hdu.copy_to(&mut old_file, &mut fits_file)?;

    let shape = fits_cube.shape();
    let old_axes: Vec<usize> = (0..shape.len()).collect();
    let old_mode:Vec<usize> = (1..shape.len()+1).collect();
    let new_axes: Vec<usize> = mode.iter().map(|x| x - 1).collect();
    
    // Swap the keys in the header
    for card_stub in ["CTYPE", "CRVAL", "CDELT", "CRPIX", "CUNIT"] {
        for fits_idx in 1..shape.len() + 1 {
            let old_card = card_stub.to_owned() + &fits_idx.to_string();
            let new_card = card_stub.to_owned() + &mode[fits_idx - 1].to_string();
            let head_val: String = hdu.read_key(&mut old_file, &old_card).unwrap();
            hdu.write_key(&mut fits_file, &new_card, head_val).unwrap();
            }  
    }
    hdu.write_image(&mut fits_file, &fits_cube.into_raw_vec())
}


/// Parse the mode string
/// 
/// # Arguments
/// 
/// * `mode` - The mode string
/// * `cube` - The FITS cube
/// 
/// # Returns
/// 
/// * `Result<Vec<usize>, Error>` - The mode as a vector of integers
/// 
fn parse_mode(mode: &str, cube: &ArrayD<f32>) -> Result<Vec<usize>,Error> {
    // Check that the mode is valid
    // First check that length of mode is equal to the number of axes in the cube
    if mode.len() != cube.ndim() {
        return Err(Error::Message(format!("Mode length {} does not match number of axes in cube ({})", mode.len(), cube.ndim())));
    }
    // Now check that all elements can be converted to integers
    let mut mode_int: Vec<usize> = Vec::new();
    let mode_split: Vec<&str> = mode.split("").collect();
    for m in mode_split {
        // Catch empty strings
        if m.is_empty() {
            continue;
        }
        match m.parse::<usize>() {
            Ok(m_int) => {
                mode_int.push(m_int);
            }
            Err(e) => {
                return Err(Error::Message(format!("Could not convert mode element {} to integer: {}", m, e)));
            }
        }
    }

    Ok(mode_int)
}

/// Simple program rotating the axes of a FITS cube
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The FITS file
    filename: String,
    /// Mode of rotation - a sequence of integers specifying the order of the axes
    /// (e.g. 321 for a 3D cube)
    mode: String,
    /// Overwrite the FITS file if it already exists
    #[arg(short='o', long="overwrite")]
    overwrite: bool,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let filename = args.filename;
    let out_filename = filename.replace(".fits", ".rot.fits");
    let check = check_file_exists(&out_filename, args.overwrite)?;
    let (fits_cube, mut fits_file) = read_fits_cube(&filename);

    let mode_vec = parse_mode(&args.mode, &fits_cube)?;

    println!("Original FITS cube shape: {:?}", fits_cube.shape());
    let rotated_fits_cube = rotate_fits_cube_axes(fits_cube, &mut fits_file, &mode_vec);
    println!("Rotated FITS cube shape: {:?}", rotated_fits_cube.shape());
    let _ = write_fits_cube(
        &out_filename,
        rotated_fits_cube,
        &mode_vec,
        fits_file,
        args.overwrite,
    )?;
    println!("Wrote rotated FITS cube to {}", out_filename);
    println!("Done!");
    Ok(())
}
