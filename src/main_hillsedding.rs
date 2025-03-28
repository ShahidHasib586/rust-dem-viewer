use anyhow::{Context, Result};
use clap::Parser;
use image::{GrayImage, Luma};
use show_image::{run_context, create_window, ImageInfo, ImageView, WindowOptions};

use std::{fs::File, io::{BufRead, BufReader}};

/// Simple DEM viewer for ASC files (grayscale)
#[derive(Parser)]
struct Args {
    /// Path to .asc DEM file
    input_file: String,
}

fn main() -> Result<()> {
    run_context(move || {
        let args = Args::parse();
        let (dem, ncols, nrows) = read_asc_file(&args.input_file)
            .expect("Failed to parse ASC file");

        let image_data = dem_to_grayscale(&dem)
            .expect("Failed to convert DEM to grayscale");

        let image = ImageView::new(
            ImageInfo::mono8(ncols as u32, nrows as u32),
            &image_data,
        );

        let window = create_window("Grayscale DEM", WindowOptions::default())
            .expect("Could not create image window");

        window.set_image("DEM", image)
            .expect("Could not set image");

        std::thread::park(); // Keep window open
        Ok::<(), anyhow::Error>(())
    })
}

/// Read .asc DEM file and return elevation data + dimensions
fn read_asc_file(path: &str) -> anyhow::Result<(Vec<f32>, usize, usize)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let ncols: usize = lines.next().unwrap()?.split_whitespace().last().unwrap().parse()?;
    let nrows: usize = lines.next().unwrap()?.split_whitespace().last().unwrap().parse()?;
    lines.next(); // xllcorner
    lines.next(); // yllcorner
    lines.next(); // cellsize
    let nodata_value: f32 = lines.next().unwrap()?.split_whitespace().last().unwrap().parse()?;

    let mut data = Vec::with_capacity(ncols * nrows);
    for line in lines {
        for val in line?.split_whitespace() {
            let v: f32 = val.parse().unwrap_or(nodata_value);
            data.push(v);
        }
    }

    Ok((data, ncols, nrows))
}

/// Convert elevation values to grayscale (u8) pixels
fn generate_hillshade(dem: &[f32], width: usize, height: usize) -> Vec<u8> {
    let mut image = vec![0u8; width * height];
    let scale = 1.0;
    let z_factor = 1.0;
    let azimuth = 315.0_f32.to_radians();
    let altitude = 45.0_f32.to_radians();
    let nodata = -99999.0;

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let center_idx = y * width + x;

            let get = |dx: isize, dy: isize| {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                    return nodata;
                }
                let i = ny as usize * width + nx as usize;
                dem[i]
            };

            let dzdx = ((get(1, -1) + 2.0 * get(1, 0) + get(1, 1)) -
                        (get(-1, -1) + 2.0 * get(-1, 0) + get(-1, 1))) / (8.0 * scale);
            let dzdy = ((get(-1, 1) + 2.0 * get(0, 1) + get(1, 1)) -
                        (get(-1, -1) + 2.0 * get(0, -1) + get(1, -1))) / (8.0 * scale);

            if dem[center_idx] == nodata {
                image[center_idx] = 0;
                continue;
            }

            let slope = (dzdx.powi(2) + dzdy.powi(2)).sqrt();
            let aspect = dzdy.atan2(-dzdx);

            let shade = (altitude.sin() * (1.0 - slope.atan()).cos() +
                         altitude.cos() * (1.0 - slope.atan()).sin() * (azimuth - aspect).cos())
                         .max(0.0);

            image[center_idx] = (shade * 255.0) as u8;
        }
    }

    image
}

