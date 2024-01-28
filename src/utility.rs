use serde::Deserialize;

use std::{error::Error, path::Path};

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub mass_distr_file_path: String,
    pub output_directory_path: String,
    pub frames_number: usize,
    pub simulation_factor: usize,
    pub target_resolution: usize,
    pub flow_field_scale: f64,
    pub dynamize_flow_field: bool,
    pub randomize_flow_field: bool,
}

impl Configuration {
    pub fn check(&self) -> Result<(), Box<dyn Error>> {
        if self.mass_distr_file_path == "" {
            return Err("Configuration Error: value of the parameter \'mass_distr_file_path\' cannot be an empty literal!".into());
        }
        if Path::new(&self.mass_distr_file_path).is_file() == false {
            return Err(format!(
                "Configuration Error: file \'{}\' does not exist!",
                self.mass_distr_file_path
            )
            .into());
        }
        if self.output_directory_path == "" {
            return Err("Configuration Error: value of the parameter \'output_directory_path\' cannot be an empty literal!".into());
        }
        if Path::new(&self.output_directory_path).is_dir() == false {
            return Err(format!(
                "Configuration Error: directory \'{}\' does not exist!",
                self.output_directory_path
            )
            .into());
        }
        if self.simulation_factor == 0 {
            return Err("Configuration Error: value of the parameter \'simulation_factor\' can not be equal to 0!".into());
        }
        if self.flow_field_scale < 1.0 {
            return Err("Configuration Error: value of the parameter \'flow_field_scale\' can not be less than 1.0!".into());
        }

        match self.target_resolution {
            480 | 720 | 1080 | 1440 | 2160 => {}
            _ => {
                return Err("Configuration Error: parameter \'target_resolution\' should take value from the set: {480, 720, 1080, 1440, 2160}!".into());
            }
        }

        return Ok(());
    }
}

#[derive(Clone)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub fn length(&self) -> f32 {
        return (self.x * self.x + self.y * self.y).sqrt();
    }
}
