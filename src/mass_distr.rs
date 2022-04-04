use image::{GenericImageView, Pixel};
use std::{error::Error, path::Path};

pub fn load_mass_distribution(
    mass_distr_file_path: &str,
    target_res_x: usize,
    target_res_y: usize,
) -> Result<Vec<f32>, Box<dyn Error>> {
    match image::open(Path::new(mass_distr_file_path)) {
        Ok(mass_distr_file) => {
            let (res_x, res_y) = mass_distr_file.dimensions();

            if (res_x as usize <= target_res_x) && (res_y as usize <= target_res_y) {
                let mut mass_distribution: Vec<f32> = vec![0.0; target_res_x * target_res_y];
                let padding_x = (target_res_x - res_x as usize) / 2;
                let padding_y = (target_res_y - res_y as usize) / 2;

                for (x, y, pixel) in mass_distr_file.pixels() {
                    mass_distribution
                        [(padding_y + y as usize) * target_res_x + padding_x + x as usize] =
                        pixel.to_luma()[0] as f32 / 255.0;
                }

                return Ok(mass_distribution);
            } else {
                return Err(format!(
                    "Error: load_mass_distribution -> \
                wskazany plik z rozkładem masy posiada rozdzielczość wyższą niż docelowa!"
                )
                .into());
            }
        }
        Err(error) => {
            return Err(error.into());
        }
    }
}
