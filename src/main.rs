use anyhow::Result;
use clap::Parser;
use image::{GrayImage, Luma};
use show_image::{run_context, create_window, ImageInfo, ImageView, WindowOptions};
use std::{fs::File, io::{BufRead, BufReader}};

/// DEM Viewer Project
#[derive(Parser)]
struct Args {
    /// Path to .asc DEM file
    input_file: String,

    /// Mode: grayscale | color | hillshade | color+hillshade
    #[clap(long, default_value = "grayscale")]
    mode: String,
}

fn main() -> Result<()> {
    run_context(move || {
        let args = Args::parse();
        let (dem, ncols, nrows) = read_asc_file(&args.input_file)?;
        let mode = args.mode;

        let image_view: ImageView<'static> = match mode.as_str() {
            "grayscale" => {
                let grayscale = dem_to_grayscale(&dem)?.into_boxed_slice();
                let leaked = Box::leak(grayscale);
                ImageView::new(ImageInfo::mono8(ncols as u32, nrows as u32), leaked)
            }
            "hillshade" => {
                let hill = generate_hillshade(&dem, ncols, nrows).into_boxed_slice();
                let leaked = Box::leak(hill);
                ImageView::new(ImageInfo::mono8(ncols as u32, nrows as u32), leaked)
            }
            "color" => {
                let color = dem_to_color_image(&dem, ncols, nrows)?.into_boxed_slice();
                let leaked = Box::leak(color);
                ImageView::new(ImageInfo::rgb8(ncols as u32, nrows as u32), leaked)
            }
            "color+hillshade" => {
                let color = dem_to_color_image(&dem, ncols, nrows)?;
                let hill = generate_hillshade(&dem, ncols, nrows);
                let blended = blend_with_hillshade(&color, &hill).into_boxed_slice();
                let leaked = Box::leak(blended);
                ImageView::new(ImageInfo::rgb8(ncols as u32, nrows as u32), leaked)
            }
            _ => panic!("Unknown mode. Use grayscale, color, hillshade, or color+hillshade"),
        };

        let window = create_window("DEM Viewer", WindowOptions::default())?;
        window.set_image("dem", image_view)?;
        std::thread::park();
        Ok::<(), anyhow::Error>(())
    })
}


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

fn dem_to_grayscale(dem: &[f32]) -> anyhow::Result<Vec<u8>> {
    let nodata_value = -99999.0;
    let valid: Vec<f32> = dem.iter().copied().filter(|&v| v != nodata_value).collect();
    let min = valid.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = valid.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let scale = 255.0 / (max - min);

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

fn dem_to_color_image(dem: &[f32], width: usize, height: usize) -> anyhow::Result<Vec<u8>> {
    let nodata = -99999.0;
    let valid: Vec<f32> = dem.iter().copied().filter(|&v| v != nodata).collect();
    let min = valid.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = valid.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    let grad = colorgrad::turbo();
    let mut rgb_image = Vec::with_capacity(width * height * 3);

    for &v in dem {
        if v == nodata {
            rgb_image.extend_from_slice(&[0, 0, 0]);
        } else {
            let norm = (v - min) / (max - min);
            let color = grad.at(norm as f64);
            rgb_image.push((color.r * 255.0) as u8);
            rgb_image.push((color.g * 255.0) as u8);
            rgb_image.push((color.b * 255.0) as u8);
        }
    }

    Ok(rgb_image)
}

fn generate_hillshade(dem: &[f32], width: usize, height: usize) -> Vec<u8> {
    let mut image = vec![0u8; width * height];
    let scale = 1.0;
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

fn blend_with_hillshade(rgb: &[u8], shade: &[u8]) -> Vec<u8> {
    rgb.chunks(3)
        .zip(shade.iter())
        .flat_map(|(color, &s)| {
            let factor = s as f32 / 255.0;
            color.iter().map(|&c| (c as f32 * factor) as u8).collect::<Vec<u8>>()
        })
        .collect()
}
