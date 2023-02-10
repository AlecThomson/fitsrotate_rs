use fitsio::{FitsFile};
use fitsio::images::{ImageType, ImageDescription};
use ndarray::{ArrayD};
use std::env;
use std::path::Path;

fn fits_index_to_array_index(fits_index: usize, naxis: usize) -> usize {
    let range = (0..(naxis as i32)).rev();
    let ret = Vec::from_iter(range)[fits_index-1];
    println!("fits_index_to_array_index: {} -> {}", fits_index, ret);
    return ret as usize;
}

fn rotate_fits_cube_axes(fits_cube: ArrayD<f32>, fits_file: &mut FitsFile) -> (ArrayD<f32>, usize) {
    let shape = fits_cube.shape();
    let axes: Vec<usize> = (0..shape.len()).collect();
    // Find the axis that corresponds to the frequency axis
    for fits_idx in 1..shape.len()+1 {
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
        }
        else {
            println!("Found axis {} with CTYPE {}", fits_idx, head_val);
        }
    }
    panic!("Could not find frequency axis");
}

fn read_fits_cube(filename: &str) -> (ArrayD<f32>, FitsFile) {
    let mut fits_file = FitsFile::open(filename).unwrap();
    let hdu = fits_file.primary_hdu().unwrap();
    let data = hdu.read_image(&mut fits_file).unwrap();
    return (data, fits_file);
}

fn write_fits_cube(filename: &str, fits_cube: ArrayD<f32>, old_spec_idx: usize, mut old_file: FitsFile, overwrite: bool) {
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

    let hdu = fits_file.hdu(0).unwrap();
    hdu.copy_to(&mut old_file, &mut fits_file).unwrap();

    let new_spec_idx: usize = 1;
    let shape = fits_cube.shape();
    // Swap the CTYPE and CRVAL for the spectral axis

    for card_stub in ["CTYPE", "CRVAL", "CDELT", "CRPIX", "CUNIT"] {
        for fits_idx in 1..shape.len()+1 {
            if fits_idx == old_spec_idx{
                let old_card = card_stub.to_owned() + &fits_idx.to_string();
                let new_card = card_stub.to_owned() + &new_spec_idx.to_string();
                let head_val: String = hdu.read_key(&mut old_file, &old_card).unwrap();
                hdu.write_key(&mut fits_file, &new_card, head_val).unwrap();
            }
            else if fits_idx == new_spec_idx {
                let old_card = card_stub.to_owned() + &fits_idx.to_string();
                let new_card = card_stub.to_owned() + &old_spec_idx.to_string();
                let head_val: String = hdu.read_key(&mut old_file, &old_card).unwrap();
                hdu.write_key(&mut fits_file, &new_card, head_val).unwrap();
            }
            else {
                continue;
            }
        }
    }
    hdu.write_image(&mut fits_file, &fits_cube.into_raw_vec()).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <filename> ", args[0]);
        return;
    }

    let filename = &args[1];
    let (fits_cube, mut fits_file) = read_fits_cube(filename);

    println!("Original FITS cube shape: {:?}", fits_cube.shape());
    let (rotated_fits_cube, old_spec_idx ) = rotate_fits_cube_axes(fits_cube.clone(), &mut fits_file);
    println!("Original FITS cube shape: {:?}", fits_cube.shape());
    println!("Rotated FITS cube shape: {:?}", rotated_fits_cube.shape());
    let out_filename = filename.replace(".fits", ".rot.fits");
    write_fits_cube(
        &out_filename,
        rotated_fits_cube,
        old_spec_idx,
        fits_file,
        true
    );
    println!("Wrote rotated FITS cube to {}", out_filename);
    println!("Done!");
}