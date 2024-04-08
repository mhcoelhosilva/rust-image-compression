#[derive(Debug, PartialEq)]
pub enum PixelUnit {
    Rgb(u8, u8, u8),
    YCbCrF32(f32, f32, f32)
}

pub fn convert_to_ycbcr_f32(pix : &PixelUnit) -> PixelUnit {
    match pix {
        PixelUnit::Rgb(r, g, b) => {
            return PixelUnit::YCbCrF32(
                (0.299*(*r as f32) + 0.587*(*g as f32) + 0.114*(*b as f32)).into(),
                (-0.169*(*r as f32) - 0.331*(*g as f32) + 0.500*(*b as f32) + 128.0).into(),
                (0.500*(*r as f32) + -0.419*(*g as f32) - 0.081*(*b as f32) + 128.0).into());
        },
        PixelUnit::YCbCrF32(y, cb, cr) => {
            return PixelUnit::YCbCrF32(*y, *cb, *cr);
        }
    }
}

pub fn convert_to_rgb(pix : &PixelUnit) -> PixelUnit {
    match pix {
        PixelUnit::Rgb(r, g, b) => {
            return PixelUnit::Rgb(*r, *g, *b);
        },
        PixelUnit::YCbCrF32(y, cb, cr) => {
            return PixelUnit::Rgb(
                (*y + 1.403*(*cr - 128.0)) as u8,
                (*y - 0.344*(*cb - 128.0) - 0.714*(*cr - 128.0)) as u8,
                (*y + 1.773*(*cb - 128.0)) as u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn RGB_to_YCbCrF32_and_back() {
        let black : PixelUnit = PixelUnit::Rgb(0, 0, 0);
        let black_ycbcr = convert_to_ycbcr_f32(&black);
        let black_converted_back = convert_to_rgb(&black_ycbcr);
        assert_eq!(black, black_converted_back);

        let red : PixelUnit = PixelUnit::Rgb(255, 0, 0);
        let red_ycbcr = convert_to_ycbcr_f32(&red);
        let red_converted_back = convert_to_rgb(&red_ycbcr);
        assert_eq!(red, red_converted_back);

        let green : PixelUnit = PixelUnit::Rgb(0, 255, 0);
        let green_ycbcr = convert_to_ycbcr_f32(&green);
        let green_converted_back = convert_to_rgb(&green_ycbcr);
        assert_eq!(green, green_converted_back);

        let blue : PixelUnit = PixelUnit::Rgb(0, 0, 255);
        let blue_ycbcr = convert_to_ycbcr_f32(&blue);
        let blue_converted_back = convert_to_rgb(&blue_ycbcr);
        assert_eq!(blue, blue_converted_back);

        let white : PixelUnit = PixelUnit::Rgb(0, 0, 0);
        let white_ycbcr = convert_to_ycbcr_f32(&white);
        let white_converted_back = convert_to_rgb(&white_ycbcr);
        assert_eq!(white, white_converted_back);
    }
}