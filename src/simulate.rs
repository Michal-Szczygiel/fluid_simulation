use crate::utility::Vec2D;

use rayon::prelude::*;

pub fn simulate(
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
