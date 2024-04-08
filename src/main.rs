extern crate nalgebra as na;

mod dct;
mod quantization;
mod image_compressor;
mod pixel_unit;

use std::env;
use std::path::Path;
use image::RgbImage;
use image::imageops::FilterType;
use na::{OMatrix, Dyn};
use std::time::Instant;
//use std::thread;
//use std::sync::mpsc;
//use std::sync::Arc;
use image::save_buffer_with_format;

use crate::dct::DCTCalculator;
use crate::quantization::QuantizationCalculator;
use crate::pixel_unit::PixelUnit;

type DMatrixf32 = OMatrix<f32, Dyn, Dyn>;

fn main() {

    let now = Instant::now();

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
            // To YCbCr, then mapped from 0-255 to -128-127
            let rgb = PixelUnit::Rgb(pixel[0], pixel[1], pixel[2]);
            let ycbcr = pixel_unit::convert_to_ycbcr_f32(&rgb);
            if let PixelUnit::YCbCrF32(y_val, cb_val, cr_val) = ycbcr {
                y[(j, i)] = y_val - 128.0;
                cb[(j, i)] = cb_val - 128.0;
                cr[(j, i)] = cr_val - 128.0;
            }
        }
    }

    //TODO: make arg
    let restricting_factor : usize = 5;
    let sub_matrix_size : usize = 8;
    let compression_percentage = 50.0_f32;

    let quant_calc = quantization::QuantizationCalculator::new(compression_percentage);
    let dct_calc = dct::DCTCalculator::new(sub_matrix_size);
    let comp = image_compressor::ImageCompressor::new(dct_calc, quant_calc);
    
    let processed_y = comp.compress_information(y.clone(), sub_matrix_size, 50.0, restricting_factor);
    let processed_cb = comp.compress_information(cb.clone(), sub_matrix_size, 50.0, restricting_factor);
    let processed_cr = comp.compress_information(cr.clone(), sub_matrix_size, 50.0, restricting_factor);

    let processed_len = processed_y.len();
    let processed_width = (buf.width() as usize) / sub_matrix_size;
    let processed_height = (buf.height() as usize) / sub_matrix_size;
    println!("Processed width = {}, processed height = {}", processed_width, processed_height);
    println!("Processed len = {}, processed_width * processed_height = {}", processed_len, processed_width * processed_height);
    let decompressed_y = comp.decompress_information(processed_y, processed_width, 50.0, restricting_factor);
    let decompressed_cb = comp.decompress_information(processed_cb, processed_width, 50.0, restricting_factor);
    let decompressed_cr = comp.decompress_information(processed_cr, processed_width, 50.0, restricting_factor);

    let real_width = decompressed_y.ncols();
    let real_height = decompressed_y.nrows();
    println!("Real width: {}, real height: {}", real_width, real_height);

    // Sanity check dimensions
    assert_eq!(real_height * real_width, processed_len * restricting_factor * restricting_factor);
    assert!(real_height * real_width < imgx * imgy);
    assert_eq!(processed_len, processed_height * processed_width);

    // To build image from raw data, we need a 1d vector with sequential (r, g, b, a) u8 values.
    let mut raw : Vec<u8> = vec![0; 4 * real_width * real_height];
    
    // index in the raw rgba array
    let mut raw_index : usize = 0;

    for i in 0..real_height {
        for j in 0..real_width {
            // map YCbCr values back from -128-127 to 0-255
            let y_val = decompressed_y[(i as usize, j as usize)] + 128.0;
            let cb_val = decompressed_cb[(i as usize, j as usize)] + 128.0;
            let cr_val = decompressed_cr[(i as usize, j as usize)] + 128.0;

            let ycbcr = PixelUnit::YCbCrF32(y_val, cb_val, cr_val);
            let rgb = pixel_unit::convert_to_rgb(&ycbcr);
            if let PixelUnit::Rgb(r, g, b) = rgb {
                raw[raw_index] = num::clamp(r as u8, 0, 255);
                raw[raw_index + 1] = num::clamp(g as u8, 0, 255);
                raw[raw_index + 2] = num::clamp(b as u8, 0, 255);
                raw[raw_index + 3] = 255;
                raw_index += 4;
            }
        }
    }

    let mut raw_uncomp : Vec<u8> = vec![0; 4 * imgx * imgy];
    let mut raw_index_uncomp : usize = 0;
    for i in 0..imgx {
        for j in 0..imgy {
            let y_val = y[(i as usize, j as usize)] + 128.0;
            let cb_val = cb[(i as usize, j as usize)] + 128.0;
            let cr_val = cr[(i as usize, j as usize)] + 128.0;

            let ycbcr = PixelUnit::YCbCrF32(y_val, cb_val, cr_val);
            let rgb = pixel_unit::convert_to_rgb(&ycbcr);
            if let PixelUnit::Rgb(r, g, b) = rgb {
                raw_uncomp[raw_index_uncomp] = num::clamp(r as u8, 0, 255);
                raw_uncomp[raw_index_uncomp + 1] = num::clamp(g as u8, 0, 255);
                raw_uncomp[raw_index_uncomp + 2] = num::clamp(b as u8, 0, 255);
                raw_uncomp[raw_index_uncomp + 3] = 255;
                raw_index_uncomp += 4;
            }
        }
    }

    save_buffer_with_format("./test/out.png", &raw, real_width.try_into().unwrap(), real_height.try_into().unwrap(), image::ColorType::Rgba8, image::ImageFormat::Png).unwrap();
    save_buffer_with_format("./test/out_uncomp.png", &raw_uncomp, imgx.try_into().unwrap(), imgy.try_into().unwrap(), image::ColorType::Rgba8, image::ImageFormat::Png).unwrap();

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}