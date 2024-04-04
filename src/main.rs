extern crate nalgebra as na;

use std::env;
use std::path::Path;
use image::{Pixel};
use image::imageops::FilterType;
use na::{OMatrix, Dyn};

mod dct;
mod quantization;

type DMatrixf32 = OMatrix<f32, Dyn, Dyn>;

// This function takes one image channel as the input and returns the 
// DCT processed output for the same.
// We use DCT on the Image to convert it from Spectral Domain 
// (Y-Cb-Cr Channels) to its equivalent Frequency Domain.

fn compressed_information(image_channel : DMatrixf32, sub_matrix_size : usize, compression_percentage : f32, restricting_factor : u8) -> Vec<DMatrixf32> {
    let h = image_channel.nrows();
    let w = image_channel.ncols(); 
    let mut output = Vec::<DMatrixf32>::new();
    for i in (0..h-sub_matrix_size).step_by(sub_matrix_size) {
        for j in (0..w-sub_matrix_size).step_by(sub_matrix_size) {
            let sub_matrix = image_channel.view((i, j), (sub_matrix_size, sub_matrix_size));
            let d = dct::calculate_dct(sub_matrix.into(), sub_matrix_size);
            let mut c = quantization::quantized_outputs(d, compression_percentage);
            c = c.view((0, 0), (restricting_factor.into(), restricting_factor.into())).into(); // 2D, 5x5
            output.push(c); // plopped onto 3D
            
            // we do this (w/N)*(h/N) = (500/8)*(500/8) = 3906 times
            // so basically the concatenated info is the compressed data for
            // each of the (w/N)*(h/N) matrices (N*N each)
            // but we are only taking restricting_factorxrestricting_factor submatrix of it
            // because we are only interested in low frequency values
        }
    }

    output
}

fn main() {
    let file : String;
    let resize_height : u32;
    let resize_width : u32;
    if env::args().count() == 4 {
        file = env::args().nth(1).unwrap();
        let resize_height_result = env::args().nth(2).unwrap().trim().parse();
        resize_height = match resize_height_result {
            Ok(s) => s,
            Err(error) => panic!("Height should be integer: {:?}", error),
        };
        let resize_width_result = env::args().nth(3).unwrap().trim().parse();
        resize_width = match resize_width_result {
            Ok(s) => s,
            Err(error) => panic!("Width should be integer: {:?}", error),
        };
    } else {
        panic!("Usage: cargo run image_path resize_height resize_width")
    };

    let im = image::open(Path::new(&file)).unwrap();
    let buf = image::imageops::resize(&im, resize_width, resize_height, FilterType::Gaussian);
    //let raw = buf.into_raw();

    let imgx = buf.width() as usize;
    let imgy = buf.height() as usize;

    let mut y = DMatrixf32::zeros(imgx as usize, imgy as usize);
    let mut cb = DMatrixf32::zeros(imgx as usize, imgy as usize);
    let mut cr = DMatrixf32::zeros(imgx as usize, imgy as usize);

    for i in 0..imgx {
        for j in 0..imgy {
            let pixel = buf.get_pixel(i as u32, j as u32);
            let luma = pixel.to_luma()[0] as f32;
            // To YCbCr, then mapped from 0-255 to -128-128
            y[(i, j)] = (16.0 + (235.0 - 16.0) * luma / 255.0 - 128.0).into();
            cb[(i, j)] = (16.0 + (240.0 - 16.0) * (pixel[2] as f32 - luma) / 255.0 - 128.0).into();
            cr[(i, j)] = (16.0 + (240.0 - 16.0) * (pixel[0] as f32 - luma) / 255.0 - 128.0).into();
        }
    }

    let processed_y = compressed_information(y, 8, 50.0, 5);
    let processed_cb = compressed_information(cb, 8, 50.0, 5);
    let processed_cr = compressed_information(cr, 8, 50.0, 5);
}