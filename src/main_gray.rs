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
fn dem_to_grayscale(dem: &[f32]) -> anyhow::Result<Vec<u8>> {
    let nodata_value = -99999.0;

    let valid: Vec<f32> = dem.iter().copied().filter(|&v| v != nodata_value).collect();

    let min = valid.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = valid.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let scale = 255.0 / (max - min);

    println!("Filtered DEM min: {}, max: {}", min, max); // helpful debug

    let image: Vec<u8> = dem
        .iter()
        .map(|&v| {
            if v == nodata_value {
                0
            } else {
                ((v - min) * scale).clamp(0.0, 255.0) as u8
            }
        })
        .collect();

    Ok(image)
}
