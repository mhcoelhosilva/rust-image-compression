extern crate nalgebra as na;

use na::{OMatrix, Dyn};
use std::f32;

type DMatrixf32 = OMatrix<f32, Dyn, Dyn>;

fn generate_n_point_dct(n : usize) -> DMatrixf32 {
    let mut d = DMatrixf32::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            if i == 0 {
                d[(i, j)] = 1.0/((n as f32).sqrt());
            } else {
                d[(i, j)] = (2.0/((n as f32).sqrt())) * (((2.0*(j as f32) + 1.0)*(i as f32)*f32::consts::PI)/(2.0*(n as f32))).cos();
            }
        }
    }
    d
}

       
// This function calculates the DCT transform using the DCT Matrix using formula 5 (pg 3 in the paper).
pub fn calculate_dct(a : DMatrixf32, n : usize) -> DMatrixf32 {
    let dct_matrix = generate_n_point_dct(n);
    let t = dct_matrix.clone().transpose();
    dct_matrix * a * t
}