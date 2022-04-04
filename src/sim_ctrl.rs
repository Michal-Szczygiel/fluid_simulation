use crate::flow_field::generate_flow_field;
use crate::mass_distr::load_mass_distribution;
use crate::save_frame::save_frame;
use crate::simulate::simulate;
use crate::utility::{Configuration, Vec2D};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use serde_json;
use std::{error::Error, fs::File, path::Path};

pub fn run(configuration_file_path: &str) -> Result<(), Box<dyn Error>> {
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
                        offset_z + (frame * config.simulation_factor + step) as f64 * 0.3,
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
                        (frame * config.simulation_factor + step) as f64 * 0.3,
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
