use crate::utility::Vec2D;

use noise::{NoiseFn, SuperSimplex};
use rayon::prelude::*;

pub fn generate_flow_field(
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
}
