extern crate nalgebra as na;

use na::{OMatrix, Dyn, dmatrix};
use std::f32;
//use num::clamp;
use std::collections::HashMap;

type DMatrixf32 = OMatrix<f32, Dyn, Dyn>;

pub struct QuantizationCalculator {
    quantization_matrices : HashMap<u32, DMatrixf32>
}

impl QuantizationCalculator {

    pub fn new(required_quality_level : f32) -> Self {
        let mut quant_calc = QuantizationCalculator { quantization_matrices : HashMap::new() };
        let quant_matrix = quant_calc.get_quantization_matrix(required_quality_level);
        quant_calc.quantization_matrices.insert(required_quality_level as u32, quant_matrix);
        quant_calc
    }

    fn generate_quantization_matrix(&self, required_quality_level : f32) -> DMatrixf32 {
        let mut q = dmatrix![16.0_f32, 11.0_f32, 10.0_f32, 16.0_f32, 24_f32, 40_f32, 51_f32, 61_f32;
                         12.0_f32, 12.0_f32, 14.0_f32, 19.0_f32, 26_f32, 58_f32, 60_f32, 55_f32;
                         14.0_f32, 13.0_f32, 16.0_f32, 24.0_f32, 40_f32, 57_f32, 69_f32, 56_f32;
                         14.0_f32, 17.0_f32, 22.0_f32, 29.0_f32, 51_f32, 87_f32, 80_f32, 62_f32;
                         18.0_f32, 22.0_f32, 37.0_f32, 56.0_f32, 68_f32, 109_f32, 103_f32, 77_f32;
                         24.0_f32, 35.0_f32, 55.0_f32, 64.0_f32, 81_f32, 104_f32, 113_f32, 92_f32;
                         49.0_f32, 64.0_f32, 78.0_f32, 87.0_f32, 103_f32, 121_f32, 120_f32, 101_f32;
                         72.0_f32, 92.0_f32, 95.0_f32, 98.0_f32, 112_f32, 100_f32, 103_f32, 99_f32];

        if required_quality_level > 50.0 {
            q = q * ((100.0 - required_quality_level)/50.0);
            //Q = np.where(Q>255,255,Q)
            //q = q.iter().map(|i| {num::clamp(i, 0, 255)}).collect();
        } else {
            q = q * (50.0/required_quality_level);
            //q = q.iter().map(|i| {num::clamp(i, 0, 255)}).collect();
        }

        q
    }

    fn get_quantization_matrix(&self, required_quality_level : f32) -> DMatrixf32 {

        let qual = required_quality_level as u32;
        if self.quantization_matrices.contains_key(&qual) {
            return self.quantization_matrices.get(&qual).unwrap().clone();
        }

        let q = self.generate_quantization_matrix(required_quality_level);

        q
    }

    //This function gives us the quantized output which can be used to 
    //find the relevant compressions in the image.
    pub fn quantize(&self, d : DMatrixf32, required_quality_level : f32) -> DMatrixf32 {
        let q = self.get_quantization_matrix(required_quality_level);
        let mut q_inv = q.try_inverse().unwrap();
        if d.ncols() < q_inv.nrows()
        {
            q_inv = q_inv.view((0, 0), (d.ncols(), d.ncols())).into();
        }
        assert_eq!(d.ncols(), q_inv.nrows());
        d * q_inv
    }

    pub fn dequantize(&self, c : DMatrixf32, required_quality_level : f32) -> DMatrixf32 {
        let mut q = self.get_quantization_matrix(required_quality_level);
        if q.ncols() > c.nrows()
        {
            q = q.view((0, 0), (c.nrows(), c.nrows())).into();
        }
        assert_eq!(q.ncols(), c.nrows());
        q * c
    }
}