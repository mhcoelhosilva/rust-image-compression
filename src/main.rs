extern crate nalgebra as na;

mod dct;
mod quantization;
mod image_compressor;

use std::env;
use std::path::Path;
use image::RgbImage;
use image::imageops::FilterType;
use na::{OMatrix, Dyn};

use crate::dct::DCTCalculator;
use crate::quantization::QuantizationCalculator;

type DMatrixf32 = OMatrix<f32, Dyn, Dyn>;

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

    let imgx = buf.width() as usize;
    let imgy = buf.height() as usize;

    let mut y = DMatrixf32::zeros(imgx as usize, imgy as usize);
    let mut cb = DMatrixf32::zeros(imgx as usize, imgy as usize);
    let mut cr = DMatrixf32::zeros(imgx as usize, imgy as usize);

    for i in 0..imgx {
        for j in 0..imgy {
            let pixel = buf.get_pixel(i as u32, j as u32);
            // To YCbCr, then mapped from 0-255 to -128-128
            y[(i, j)] = (0.299*(pixel[0] as f32) + 0.587*(pixel[1] as f32) + 0.114*(pixel[2] as f32) - 128.0).into();
            cb[(i, j)] = (128.0 - 0.169*(pixel[0] as f32) - 0.331*(pixel[1] as f32) + 0.500*(pixel[2] as f32) - 128.0).into();
            cr[(i, j)] = (128.0 + 0.500*(pixel[0] as f32) + -0.419*(pixel[1] as f32) - 0.081*(pixel[2] as f32) - 128.0).into();
        }
    }

    //TODO: make arg
    let restricting_factor : usize = 5; 
    let sub_matrix_size : usize = 8;
    let compression_percentage = 50.0_f32;

    let quant_calc = quantization::QuantizationCalculator::new(compression_percentage);
    let dct_calc = dct::DCTCalculator::new(sub_matrix_size);
    let mut comp = image_compressor::ImageCompressor::new(dct_calc, quant_calc);
    
    let processed_y = comp.compress_information(y, sub_matrix_size, 50.0, restricting_factor);
    let processed_cb = comp.compress_information(cb, sub_matrix_size, 50.0, restricting_factor);
    let processed_cr = comp.compress_information(cr, sub_matrix_size, 50.0, restricting_factor);

    let processed_width = (buf.width() as usize) / sub_matrix_size;
    let decompressed_y = comp.decompress_information(processed_y, processed_width, 50.0, restricting_factor);
    let decompressed_cb = comp.decompress_information(processed_cb, processed_width, 50.0, restricting_factor);
    let decompressed_cr = comp.decompress_information(processed_cr, processed_width, 50.0, restricting_factor);

    // To build image from raw data, we need a 1d vector with sequential (r, g, b, a) u8 values.
    let real_width = decompressed_y.ncols();
    let real_height = decompressed_y.nrows();
    let mut raw : Vec<u8> = vec![0; 4 * real_width * real_height];
    
    // index in the raw rgba array
    let mut raw_index : usize = 0;

    for i in 0..real_height {
        for j in 0..real_width {
            let y_val = decompressed_y[(i as usize, j as usize)] + 128.0;
            let cb_val = decompressed_cb[(i as usize, j as usize)] + 128.0;
            let cr_val = decompressed_cr[(i as usize, j as usize)] + 128.0;

            let r_val = y_val + 1.403*(cr_val - 128.0);
            let g_val = y_val - 0.344*(cb_val - 128.0) - 0.714*(cr_val - 128.0);
            let b_val = y_val + 1.773*(cb_val - 128.0);

            //assert!(r_val <= 255.0);
            //assert!(g_val <= 255.0);
            //assert!(b_val <= 255.0);

            raw[raw_index] = num::clamp(r_val as u8, 0, 255);
            raw[raw_index + 1] = num::clamp(g_val as u8, 0, 255);
            raw[raw_index + 2] = num::clamp(b_val as u8, 0, 255);
            raw[raw_index + 3] = 255;
            raw_index += 4;
        }
    }

    let image_out = RgbImage::from_raw(real_width.try_into().unwrap(), real_height.try_into().unwrap(), raw)
        .expect("container should have the right size for the image dimensions");

    println!("Saving decompressed image out to ./test/out.png...");
    let _ = image_out.save("./test/out.png");
}