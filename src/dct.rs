extern crate nalgebra as na;

use na::{OMatrix, Dyn};
use std::f32;
use std::collections::HashMap;

type DMatrixf32 = OMatrix<f32, Dyn, Dyn>;

pub struct DCTCalculator {
    dct_matrices : HashMap<usize, DMatrixf32>
}

impl DCTCalculator {

    pub fn new(n : usize) -> Self {
        let mut dct_cal = DCTCalculator { dct_matrices : HashMap::new() };
        let dct_matrix = dct_cal.generate_n_point_dct(n);
        dct_cal.dct_matrices.insert(n, dct_matrix);
        dct_cal
    }

    fn generate_n_point_dct(&self, n : usize) -> DMatrixf32 {
        let mut d = DMatrixf32::zeros(n, n);
        for i in 0..n {
            for j in 0..n {
                if i == 0 {
                    d[(i, j)] = 1.0/((n as f32).sqrt());
                } else {
                    d[(i, j)] = (2.0/(n as f32)).sqrt() * (((2.0*(j as f32) + 1.0)*(i as f32)*f32::consts::PI)/(2.0*(n as f32))).cos();
                }
            }
        }

        d
    }

    fn get_n_point_dct(&self, n : usize) -> DMatrixf32 {
        if self.dct_matrices.contains_key(&n) {
            return self.dct_matrices.get(&n).unwrap().clone();
        }

        let mut d = self.generate_n_point_dct(n);

        d
    }

    // This function calculates the DCT transform using the DCT Matrix using formula 5 (pg 3 in the paper).
    pub fn calculate_dct(&self, a : DMatrixf32, n : usize) -> DMatrixf32 {
        let dct_matrix = self.get_n_point_dct(n);
        let t = dct_matrix.clone().transpose();
        assert_eq!(dct_matrix.ncols(), a.nrows());
        let temp = dct_matrix * a;
        assert_eq!(temp.ncols(), t.nrows());
        temp * t // dct * a * dct_transpose
    }

    pub fn calculate_inverse_dct(&self, a : DMatrixf32, n : usize) -> DMatrixf32 {
        let dct_matrix = self.get_n_point_dct(n);
        let t = dct_matrix.clone().transpose();
        assert_eq!(t.ncols(), a.nrows());
        let temp = t * a;
        assert_eq!(temp.ncols(), dct_matrix.nrows());
        temp * dct_matrix // dc_transpose * a * dct
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverse() {
        let sub_matrix_size = 8;
        let dct_calc = DCTCalculator::new(sub_matrix_size);
        let sub_matrix = DMatrixf32::identity(sub_matrix_size, sub_matrix_size);
        let dct = dct_calc.calculate_dct(sub_matrix.clone(), sub_matrix_size);
        let inversed = dct_calc.calculate_inverse_dct(dct, sub_matrix_size);
        assert!(inversed.is_identity(0.01_f32));
    }
}