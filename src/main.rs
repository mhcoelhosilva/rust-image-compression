extern crate nalgebra as na;

use std::env;
use std::path::Path;
use image::RgbImage;
use image::imageops::FilterType;
use na::{OMatrix, Dyn};

mod dct;
mod quantization;

type DMatrixf32 = OMatrix<f32, Dyn, Dyn>;

/*
This function takes one image channel as the input and returns the 
DCT processed output for the same.
We use DCT on the Image to convert it from Spectral Domain 
(Y-Cb-Cr Channels) to its equivalent Frequency Domain.
*/
fn compressed_information(image_channel : DMatrixf32, sub_matrix_size : usize, compression_percentage : f32, restricting_factor : usize) -> Vec<DMatrixf32> {
    let h = image_channel.nrows();
    let w = image_channel.ncols(); 
    let mut output = Vec::<DMatrixf32>::new();
    for i in (0..h-sub_matrix_size).step_by(sub_matrix_size) {
        for j in (0..w-sub_matrix_size).step_by(sub_matrix_size) {
            let sub_matrix = image_channel.view((i, j), (sub_matrix_size, sub_matrix_size));
            let d = dct::calculate_dct(sub_matrix.into(), sub_matrix_size);
            let mut c = quantization::quantized_outputs(d, compression_percentage);
            c = c.view((0, 0), (restricting_factor, restricting_factor)).into(); // 2D, 5x5
            output.push(c); // plopped onto 3D
            
            /*
            This is done (w/sub_matrix_size)*(h/sub_matrix_size) times
            Each submatrix represents sub_matrix_size pixels in the original image channel,
            but we are only taking restricting_factorxrestricting_factor submatrix of it
            because we are only interested in low frequency values.
            */
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

    let restricting_factor : usize = 5; //TODO: make arg
    let sub_matrix_size : usize = 8;
    let processed_y = compressed_information(y, sub_matrix_size, 50.0, restricting_factor);
    let processed_cb = compressed_information(cb, sub_matrix_size, 50.0, restricting_factor);
    let processed_cr = compressed_information(cr, sub_matrix_size, 50.0, restricting_factor);

    let processed_len = processed_y.len(); // number of submatrices
    let processed_width = buf.width() / (sub_matrix_size as u32);
    let processed_height = buf.height() / (sub_matrix_size as u32);

    // To build image from raw data, we need a 1d vector with sequential (r, g, b, a) u8 values.
    // There are processed_len matrices, each with restricting_factor*restricting_factor Rgba values.
    let mut raw : Vec<u8> = vec![0; 4 * processed_len * restricting_factor * restricting_factor];
    let real_width = processed_width * (restricting_factor as u32);
    let real_height = processed_height * (restricting_factor as u32);

    // index in the raw rgba array
    let mut raw_index : usize = 0;

    // k : submatrix index
    for k in 0..processed_len {
        // i and j: the pixel indices inside each matrix
        for i in 0..restricting_factor {
            for j in 0..restricting_factor {
                let y_val = processed_y[k][(i as usize, j as usize)] + 128.0;
                let cb_val = processed_cb[k][(i as usize, j as usize)] + 128.0;
                let cr_val = processed_cr[k][(i as usize, j as usize)] + 128.0;

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
    }

    let image_out = RgbImage::from_raw(real_width, real_height, raw)
        .expect("container should have the right size for the image dimensions");
    let _ = image_out.save("./test/out.png");
}