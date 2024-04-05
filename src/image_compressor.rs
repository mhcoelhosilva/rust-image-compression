extern crate nalgebra as na;

use na::{OMatrix, Dyn};
use crate::DCTCalculator;
use crate::QuantizationCalculator;

type DMatrixf32 = OMatrix<f32, Dyn, Dyn>;

pub struct ImageCompressor {
    dct_calc : DCTCalculator,
    quant_calc : QuantizationCalculator
}

impl ImageCompressor {

    pub fn new(dct_calc : DCTCalculator, quant_calc : QuantizationCalculator) -> Self {
        let compressor = ImageCompressor { dct_calc, quant_calc };
        compressor
    }

    pub fn compress_information(&mut self, image_channel : DMatrixf32, sub_matrix_size : usize, compression_percentage : f32, restricting_factor : usize) -> Vec<DMatrixf32> {
        let h = image_channel.nrows();
        let w = image_channel.ncols(); 
        let mut output = Vec::<DMatrixf32>::new();
        for i in (0..h-sub_matrix_size).step_by(sub_matrix_size) {
            for j in (0..w-sub_matrix_size).step_by(sub_matrix_size) {
                let sub_matrix = image_channel.view((i, j), (sub_matrix_size, sub_matrix_size));
                let d = self.dct_calc.calculate_dct(sub_matrix.into(), sub_matrix_size);
                let mut c = self.quant_calc.quantize(d, compression_percentage);
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

    pub fn decompress_information(&mut self, sub_matrices : Vec<DMatrixf32>, new_width : usize, compression_percentage : f32, restricting_factor : usize) -> DMatrixf32 {
        
        let num_submatrices = sub_matrices.len();
        let full_matrix_size = num_submatrices * restricting_factor;
        let mut full_matrix = DMatrixf32::zeros(full_matrix_size, full_matrix_size);

        // k : submatrix index
        for k in 0..num_submatrices {
            // this is the submatrix index
            let col = k % new_width;
            let row = k / new_width;

            let sub_matrix = sub_matrices[k].clone();
            //let sub_matrix_dim = sub_matrix.nrows();
            //let sub_matrix = sub_matrix.insert_rows(sub_matrix_dim, 3, 0.0_f32);
            //let sub_matrix = sub_matrix.insert_columns(sub_matrix_dim, 3, 0.0_f32);
            let unquantized_submatrix = self.quant_calc.dequantize(sub_matrix, compression_percentage);
            let unquantized_submatrix_dim = unquantized_submatrix.nrows();
            let n = self.dct_calc.calculate_inverse_dct(unquantized_submatrix, unquantized_submatrix_dim);
            for i in 0..restricting_factor {
                for j in 0..restricting_factor {
                    full_matrix[(row*restricting_factor + i, col*restricting_factor + j)] = n[(i, j)];
                }
            }
        }
        
        full_matrix
    }
}