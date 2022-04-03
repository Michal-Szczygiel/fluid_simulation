use console::style;
use image::{
    codecs::png::PngEncoder, ColorType, GenericImageView, ImageBuffer, ImageEncoder, Pixel,
};
use indicatif::{ProgressBar, ProgressStyle};
use noise::{NoiseFn, SuperSimplex};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use rayon::prelude::*;
use serde::Deserialize;
use serde_json;
use std::{error::Error, fs::File, io::BufWriter, path::Path};

#[derive(Debug, Deserialize)]
struct Configuration {
    mass_distr_file_path: String,
    output_directory_path: String,
    frames_number: usize,
    simulation_factor: usize,
    target_resolution: usize,
    flow_field_scale: f64,
    dynamize_flow_field: bool,
    randomize_flow_field: bool,
}

impl Configuration {
    fn check(&self) -> Result<(), Box<dyn Error>> {
        if self.mass_distr_file_path == "" {
            return Err("Configuration Error: wartość parametru \'mass_distr_file_path\' nie może być literałem pustym!".into());
        }
        if Path::new(&self.mass_distr_file_path).is_file() == false {
            return Err(format!(
                "Configuration Error: plik \'{}\' nie istnieje!",
                self.mass_distr_file_path
            )
            .into());
        }
        if self.output_directory_path == "" {
            return Err("Configuration Error: wartość parametru \'output_directory_path\' nie może być literałem pustym!".into());
        }
        if Path::new(&self.output_directory_path).is_dir() == false {
            return Err(format!(
                "Configuration Error: katalog \'{}\' nie istnieje!",
                self.output_directory_path
            )
            .into());
        }
        if self.simulation_factor == 0 {
            return Err("Configuration Error: wartość parametru \'simulation_factor\' nie może być równa 0!".into());
        }
        if self.flow_field_scale < 1.0 {
            return Err("Configuration Error: wartość parametru \'flow_field_scale\' nie może być mniejsza od 1.0!".into());
        }

        match self.target_resolution {
            480 | 720 | 1080 | 1440 | 2160 => {}
            _ => {
                return Err("Configuration Error: parametr \'target_resolution\' powinien przyjmować wartości ze zbioru {480, 720, 1080, 1440, 2160}!".into());
            }
        }

        return Ok(());
    }
}

#[derive(Clone)]
struct Vec2D {
    x: f32,
    y: f32,
}

impl Vec2D {
    fn length(&self) -> f32 {
        return (self.x * self.x + self.y * self.y).sqrt();
    }
}

fn load_mass_distribution(
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

fn generate_flow_field(
    flow_field: &mut Vec<Vec2D>,
    noise_buffer: &mut Vec<f32>,
    res_x: usize,
    res_y: usize,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
    offset_z: f64,
) {
    let noise = SuperSimplex::new();
    let mut max_magnitude: f32 = f32::MIN;

    noise_buffer
        .par_chunks_mut(res_x)
        .enumerate()
        .for_each(|(y, chunk)| {
            for (x, value) in chunk.iter_mut().enumerate() {
                *value = noise.get([
                    (x as f64 - offset_x) / scale,
                    (y as f64 - offset_y) / scale,
                    offset_z / scale,
                ]) as f32;
            }
        });

    flow_field
        .par_chunks_mut(res_x)
        .enumerate()
        .skip(1)
        .take(res_y - 2)
        .for_each(|(y, chunk)| {
            for (x, value) in chunk.iter_mut().enumerate().skip(1).take(res_x - 2) {
                *value = Vec2D {
                    x: (noise_buffer[(y + 1) * res_x + x] - noise_buffer[(y - 1) * res_x + x])
                        as f32,
                    y: -(noise_buffer[y * res_x + x + 1] - noise_buffer[y * res_x + x - 1]) as f32,
                };
            }
        });

        for value in flow_field.iter() {
            max_magnitude = max_magnitude.max(value.length());
        }

        flow_field.par_iter_mut().for_each(|value| {
            value.x = value.x / max_magnitude;
            value.y = value.y / max_magnitude;
        });

    /*
    for y in 0..res_y {
        for x in 0..res_x {
            noise_buffer[y * res_x + x] = noise.get([
                (x as f64 - offset_x) / scale,
                (y as f64 - offset_y) / scale,
                offset_z / scale,
            ]) as f32;
        }
    }

    for y in 1..res_y - 1 {
        for x in 1..res_x - 1 {
            flow_field[y * res_x + x] = Vec2D {
                x: (noise_buffer[(y + 1) * res_x + x] - noise_buffer[(y - 1) * res_x + x]) as f32,
                y: -(noise_buffer[y * res_x + x + 1] - noise_buffer[y * res_x + x - 1]) as f32,
            };

            max_magnitude = max_magnitude.max(flow_field[y * res_x + x].length());
        }
    }

    for vec in flow_field.iter_mut() {
        (*vec).x /= max_magnitude;
        (*vec).y /= max_magnitude;
    }
    */
}

fn simulate(
    flow_field: &Vec<Vec2D>,
    mass_distr: &mut Vec<f32>,
    mass_buffer: &mut Vec<f32>,
    res_x: usize,
    res_y: usize,
) {
    mass_buffer
        .par_chunks_mut(res_x)
        .enumerate()
        .skip(1)
        .take(res_y - 2)
        .for_each(|(y, chunk)| {
            let mut grad = Vec2D { x: 0.0, y: 0.0 };
            let mut diff: f32;

            for (x, value) in chunk.iter_mut().enumerate().skip(1).take(res_x - 2) {
                grad.x = if flow_field[y * res_x + x].x < 0.0 {
                    mass_distr[y * res_x + x + 1] - mass_distr[y * res_x + x]
                } else {
                    mass_distr[y * res_x + x] - mass_distr[y * res_x + x - 1]
                };

                grad.y = if flow_field[y * res_x + x].y < 0.0 {
                    mass_distr[(y + 1) * res_x + x] - mass_distr[y * res_x + x]
                } else {
                    mass_distr[y * res_x + x] - mass_distr[(y - 1) * res_x + x]
                };

                diff = flow_field[y * res_x + x].x * grad.x + flow_field[y * res_x + x].y * grad.y;
                *value = mass_distr[y * res_x + x] - 0.5 * diff;
            }
        });

    /*
    let mut grad = Vec2D { x: 0.0, y: 0.0 };
    let mut diff: f32;

    for y in 1..res_y - 1 {
        for x in 1..res_x - 1 {
            grad.x = if flow_field[y * res_x + x].x < 0.0 {
                mass_distr[y * res_x + x + 1] - mass_distr[y * res_x + x]
            } else {
                mass_distr[y * res_x + x] - mass_distr[y * res_x + x - 1]
            };

            grad.y = if flow_field[y * res_x + x].y < 0.0 {
                mass_distr[(y + 1) * res_x + x] - mass_distr[y * res_x + x]
            } else {
                mass_distr[y * res_x + x] - mass_distr[(y - 1) * res_x + x]
            };

            diff = flow_field[y * res_x + x].x * grad.x + flow_field[y * res_x + x].y * grad.y;
            mass_buffer[y * res_x + x] = mass_distr[y * res_x + x] - 0.5 * diff;
        }
    }
    */

    std::mem::swap(mass_distr, mass_buffer);
}

fn save_frame(
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

fn run(configuration_file_path: &str) -> Result<(), Box<dyn Error>> {
    let config_file = match File::open(Path::new(configuration_file_path)) {
        Ok(file) => file,
        Err(error) => {
            return Err(format!(
                "Configuration File Error: nie można otworzyć pliku konfiguracyjnego! \
            Szczegóły: {}",
                error
            )
            .into());
        }
    };

    let config: Configuration = match serde_json::from_reader(config_file) {
        Ok(config) => config,
        Err(error) => {
            return Err(format!(
                "Configuration File Error: nie można odczytać zawartości pliku konfiguracyjnego! \
            Szczegóły: {}",
                error
            )
            .into());
        }
    };

    match config.check() {
        Ok(_) => {}
        Err(error) => {
            return Err(error.into());
        }
    };

    println!(
        "{}\n  - mass_distr_file_path:   {}\n  - output_directory_path:  {}\n  - frames_number:          {}\
        \n  - simulation_factor:      {}\n  - target_resolution:      {}\n  - flow_field_scale:       {}\
        \n  - dynamize_flow_field:    {}\n  - randomize_flow_field:   {}\n",
        style("Rozpoczynam symulację z następującymi parametrami:")
            .bold()
            .underlined()
            .green(),
        style(format!("\'{}\'", config.mass_distr_file_path)).bold().blue(),
        style(format!("\'{}\'", config.output_directory_path)).bold().blue(),
        style(&config.frames_number).bold().blue(),
        style(format!("{}x", config.simulation_factor)).bold().blue(),
        style(format!("{}p", config.target_resolution)).bold().blue(),
        style(&config.flow_field_scale).bold().blue(),
        style(&config.dynamize_flow_field).bold().blue(),
        style(&config.randomize_flow_field).bold().blue()
    );

    let res_x: usize;
    let res_y: usize;

    match config.target_resolution {
        480 => {
            res_x = 640;
            res_y = 460;
        }
        720 => {
            res_x = 1280;
            res_y = 720;
        }
        1080 => {
            res_x = 1920;
            res_y = 1080;
        }
        1440 => {
            res_x = 2560;
            res_y = 1440;
        }
        2160 => {
            res_x = 3840;
            res_y = 2160;
        }
        _ => {
            unreachable!();
        }
    }

    let mut mass_distr = match load_mass_distribution(&config.mass_distr_file_path, res_x, res_y) {
        Ok(mass_distr) => mass_distr,
        Err(error) => {
            return Err(error.into());
        }
    };

    let mut mass_buffer: Vec<f32> = vec![0.0; res_x * res_y];
    let mut flow_field: Vec<Vec2D> = vec![Vec2D { x: 0.0, y: 0.0 }; res_x * res_y];
    let mut noise_buffer: Vec<f32> = vec![0.0; res_x * res_y];

    let bar = ProgressBar::new(config.frames_number as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:70.cyan/blue}] {pos:>7}/{len:7} {msg}")
            .progress_chars("#>-"),
    );

    if config.randomize_flow_field == true {
        let mut rng = thread_rng();
        let distr = Uniform::new(-5.0, 5.0);

        let offset_x = distr.sample(&mut rng);
        let offset_y = distr.sample(&mut rng);
        let offset_z = distr.sample(&mut rng);

        if config.dynamize_flow_field == true {
            for frame in 0..config.frames_number {
                for step in 0..config.simulation_factor {
                    generate_flow_field(
                        &mut flow_field,
                        &mut noise_buffer,
                        res_x,
                        res_y,
                        config.flow_field_scale,
                        offset_x,
                        offset_y,
                        offset_z + (frame * config.simulation_factor + step) as f64,
                    );

                    simulate(&flow_field, &mut mass_distr, &mut mass_buffer, res_x, res_y)
                }

                match save_frame(
                    &format!("{}/frame_{}.png", config.output_directory_path, frame),
                    &mass_distr,
                    res_x,
                    res_y,
                    170.0,
                    0.03,
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(error.into());
                    }
                }

                bar.inc(1);
            }

            bar.finish();
        } else {
            generate_flow_field(
                &mut flow_field,
                &mut noise_buffer,
                res_x,
                res_y,
                config.flow_field_scale,
                offset_x,
                offset_y,
                offset_z,
            );

            for frame in 0..config.frames_number {
                for _ in 0..config.simulation_factor {
                    simulate(&flow_field, &mut mass_distr, &mut mass_buffer, res_x, res_y)
                }

                match save_frame(
                    &format!("{}/frame_{}.png", config.output_directory_path, frame),
                    &mass_distr,
                    res_x,
                    res_y,
                    170.0,
                    0.03,
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(error.into());
                    }
                }

                bar.inc(1);
            }
        }

        bar.finish();
    } else {
        if config.dynamize_flow_field == true {
            for frame in 0..config.frames_number {
                for step in 0..config.simulation_factor {
                    generate_flow_field(
                        &mut flow_field,
                        &mut noise_buffer,
                        res_x,
                        res_y,
                        config.flow_field_scale,
                        0.0,
                        0.0,
                        (frame * config.simulation_factor + step) as f64,
                    );

                    simulate(&flow_field, &mut mass_distr, &mut mass_buffer, res_x, res_y)
                }

                match save_frame(
                    &format!("{}/frame_{}.png", config.output_directory_path, frame),
                    &mass_distr,
                    res_x,
                    res_y,
                    170.0,
                    0.03,
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(error.into());
                    }
                }

                bar.inc(1);
            }

            bar.finish();
        } else {
            generate_flow_field(
                &mut flow_field,
                &mut noise_buffer,
                res_x,
                res_y,
                config.flow_field_scale,
                0.0,
                0.0,
                0.0,
            );

            for frame in 0..config.frames_number {
                for _ in 0..config.simulation_factor {
                    simulate(&flow_field, &mut mass_distr, &mut mass_buffer, res_x, res_y)
                }

                match save_frame(
                    &format!("{}/frame_{}.png", config.output_directory_path, frame),
                    &mass_distr,
                    res_x,
                    res_y,
                    170.0,
                    0.03,
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(error.into());
                    }
                }

                bar.inc(1);
            }

            bar.finish();
        }
    }

    return Ok(());
}

fn main() {
    let wojak = r#"
      ⠘⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡜⠀⠀⠀
    ⠀⠀⠀⠑⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡔⠁⠀⠀⠀
    ⠀⠀⠀⠀⠈⠢⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠴⠊⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⢀⣀⣀⣀⣀⣀⡀⠤⠄⠒⠈⠀⠀⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⠀⠘⣀⠄⠊⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡠⠔⠒⠒⠒⠒⠒⠢⠤⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⠀⡰⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⢄⡀⠀⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⡸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡀⠀⠀⠀⠀⠙⠄⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⢀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠃⠀⢠⠂⠀⠀⠘⡄⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠈⢤⡀⢂⠀⢨⠀⢀⡠⠈⢣⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⢀⢀⡖⠒⠶⠤⠭⢽⣟⣗⠲⠖⠺⣖⣴⣆⡤⠤⠤⠼⡄⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠘⡈⠃⠀⠀⠀⠘⣺⡟⢻⠻⡆⠀⡏⠀⡸⣿⢿⢞⠄⡇⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⢣⡀⠤⡀⡀⡔⠉⣏⡿⠛⠓⠊⠁⠀⢎⠛⡗⡗⢳⡏⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⠀⢱⠀⠨⡇⠃⠀⢻⠁⡔⢡⠒⢀⠀⠀⡅⢹⣿⢨⠇⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⠀⢸⠀⠠⢼⠀⠀⡎⡜⠒⢀⠭⡖⡤⢭⣱⢸⢙⠆⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⠀⡸⠀⠀⠸⢁⡀⠿⠈⠂⣿⣿⣿⣿⣿⡏⡍⡏⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⢀⠇⠀⠀⠀⠀⠸⢢⣫⢀⠘⣿⣿⡿⠏⣼⡏⠀⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⣀⣠⠊⠀⣀⠎⠁⠀⠀⠀⠙⠳⢴⡦⡴⢶⣞⣁⣀⣀⡀⠀⠀⠀⠀⠀
    ⠀⠐⠒⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⠀⢀⠤⠀⠀⠀⠀⠀⠀⠀⠈⠉⠀⠀⠀⠀"#;

    let pikachu = r#"
    ⠸⣷⣦⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣠⣤⠀⠀⠀
    ⠀⠙⣿⡄⠈⠑⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⠔⠊⠉⣿⡿⠁⠀⠀⠀
    ⠀⠀⠈⠣⡀⠀⠀⠑⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠊⠁⠀⠀⣰⠟⠀⠀⠀⣀⣀
    ⠀⠀⠀⠀⠈⠢⣄⠀⡈⠒⠊⠉⠁⠀⠈⠉⠑⠚⠀⠀⣀⠔⢊⣠⠤⠒⠊⠉⠀⡜
    ⠀⠀⠀⠀⠀⠀⠀⡽⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠩⡔⠊⠁⠀⠀⠀⠀⠀⠀⠇
    ⠀⠀⠀⠀⠀⠀⠀⡇⢠⡤⢄⠀⠀⠀⠀⠀⡠⢤⣄⠀⡇⠀⠀⠀⠀⠀⠀⠀⢰⠀
    ⠀⠀⠀⠀⠀⠀⢀⠇⠹⠿⠟⠀⠀⠤⠀⠀⠻⠿⠟⠀⣇⠀⠀⡀⠠⠄⠒⠊⠁⠀
    ⠀⠀⠀⠀⠀⠀⢸⣿⣿⡆⠀⠰⠤⠖⠦⠴⠀⢀⣶⣿⣿⠀⠙⢄⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⠀⢻⣿⠃⠀⠀⠀⠀⠀⠀⠀⠈⠿⡿⠛⢄⠀⠀⠱⣄⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⠀⢸⠈⠓⠦⠀⣀⣀⣀⠀⡠⠴⠊⠹⡞⣁⠤⠒⠉⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⣠⠃⠀⠀⠀⠀⡌⠉⠉⡤⠀⠀⠀⠀⢻⠿⠆⠀⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠰⠁⡀⠀⠀⠀⠀⢸⠀⢰⠃⠀⠀⠀⢠⠀⢣⠀⠀⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⢶⣗⠧⡀⢳⠀⠀⠀⠀⢸⣀⣸⠀⠀⠀⢀⡜⠀⣸⢤⣶⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠈⠻⣿⣦⣈⣧⡀⠀⠀⢸⣿⣿⠀⠀⢀⣼⡀⣨⣿⡿⠁⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠈⠻⠿⠿⠓⠄⠤⠘⠉⠙⠤⢀⠾⠿⣿⠟⠋"#;

    let args: Vec<String> = std::env::args().collect();
    let config_file_path = match args.get(1) {
        Some(path) => path,
        None => {
            eprintln!(
                "{}\n\n{}\n",
                style("Podaj ścieżkę do pliku konfiguracyjnego...")
                    .bold()
                    .yellow(),
                wojak
            );
            return;
        }
    };

    match run(config_file_path) {
        Ok(_) => {
            let message = style("Zakończono symulację!").bold().green();
            println!("{}\n\n{}\n", message, pikachu);
        }
        Err(error) => {
            let message = style(format!("{}", error)).bold().red();
            eprintln!("{}\n\n{}\n", message, wojak);
        }
    }
}
