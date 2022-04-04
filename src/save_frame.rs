use image::{codecs::png::PngEncoder, ColorType, ImageBuffer, ImageEncoder};
use std::{error::Error, fs::File, io::BufWriter, path::Path};

pub fn save_frame(
    image_file_path: &str,
    buffer: &Vec<f32>,
    res_x: usize,
    res_y: usize,
    midpoint: f32,
    slope: f32,
) -> Result<(), Box<dyn Error>> {
    let frame = ImageBuffer::from_fn(res_x as u32, res_y as u32, |x, y| {
        let value = buffer[(y * res_x as u32 + x) as usize] * 255.0;
        let image_value = (255.0 / (1.0 + (-slope * (value - midpoint)).exp())).round() as u8;

        image::Luma([image_value])
    });

    match File::create(Path::new(image_file_path)) {
        Ok(output_file) => {
            let frame_encoder = PngEncoder::new(BufWriter::new(output_file));

            match frame_encoder.write_image(&frame, res_x as u32, res_y as u32, ColorType::L8) {
                Ok(_) => {
                    return Ok(());
                }
                Err(error) => {
                    return Err(error.into());
                }
            }
        }
        Err(error) => {
            return Err(error.into());
        }
    }
}
