use anyhow::Result;
use clap::Parser;
use show_image::{run_context, create_window, ImageInfo, ImageView, WindowOptions};
use std::{fs::File, io::{BufRead, BufReader}};

// This program visualizes Digital Elevation Model (DEM) data from .asc files.
// It supports various visualization modes including grayscale, color, hillshade, and a combination of color and hillshade.
// The program uses the `show_image` crate for displaying images and `clap` for command-line argument parsing.
// The DEM data is read from a .asc file, and the visualization is generated based on user input.

// The program is designed to be run from the command line with the following syntax:
// `cargo run -- <input_file> --mode <visualization_mode>`
// The input file should be a valid .asc DEM file, and the mode can be one of:
// "grayscale", "color", "hillshade", or "color+hillshade".


// This struct defines the command-line arguments for the DEM Viewer.
#[derive(Parser)]
struct Args {
    // Path to the .asc DEM file to be visualized.
    input_file: String,

    // Mode of visualization: grayscale, color, hillshade, or color+hillshade.
    #[clap(long, default_value = "grayscale")]
    mode: String,
}

fn main() -> Result<()> {
    // Running the application within a context that manages the image display type.
    run_context(move || {
        // Parse the command-line arguments.
        let args = Args::parse();

        // Read the DEM data from the specified input file.
        let (dem, ncols, nrows) = read_asc_file(&args.input_file)?;
        // detect the visualization mode from the command line arguments.
        let mode = args.mode;

        // Create an image view based on the selected visualization mode.
        let image_view: ImageView<'static> = match mode.as_str() {
            "grayscale" => {
                // Convert the DEM data to a grayscale image.
                let grayscale = dem_to_grayscale(&dem)?.into_boxed_slice();
                let leaked = Box::leak(grayscale);
                ImageView::new(ImageInfo::mono8(ncols as u32, nrows as u32), leaked)
            }
            "hillshade" => {
                // Generate a hillshade image from the DEM data.
                let hill = generate_hillshade(&dem, ncols, nrows).into_boxed_slice();
                let leaked = Box::leak(hill);
                ImageView::new(ImageInfo::mono8(ncols as u32, nrows as u32), leaked)
            }
            "color" => {
                // Convert the DEM data to a color image.
                let color = dem_to_color_image(&dem, ncols, nrows)?.into_boxed_slice();
                let leaked = Box::leak(color);
                ImageView::new(ImageInfo::rgb8(ncols as u32, nrows as u32), leaked)
            }
            "color+hillshade" => {
                // Blend the color image with the hillshade.
                let color = dem_to_color_image(&dem, ncols, nrows)?;
                let hill = generate_hillshade(&dem, ncols, nrows);
                let blended = blend_with_hillshade(&color, &hill).into_boxed_slice();
                let leaked = Box::leak(blended);
                ImageView::new(ImageInfo::rgb8(ncols as u32, nrows as u32), leaked)
            }
            _ => panic!("Unknown mode. Use grayscale, color, hillshade, or color+hillshade"),
        };

        // Create a window to display the image.
        let window = create_window("DEM Viewer", WindowOptions::default())?;
        window.set_image("dem", image_view)?;
        // Keep the window open.
        std::thread::park();
        Ok::<(), anyhow::Error>(())
    })
}

/// Reads the .asc file and returns the DEM data along with the number of columns and rows.
fn read_asc_file(path: &str) -> anyhow::Result<(Vec<f32>, usize, usize)> {
    // Open the file and create a buffered reader.
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Parse the header information from the file.
    let ncols: usize = lines.next().unwrap()?.split_whitespace().last().unwrap().parse()?;
    let nrows: usize = lines.next().unwrap()?.split_whitespace().last().unwrap().parse()?;
    lines.next(); // Skip xllcorner
    lines.next(); // Skip yllcorner
    lines.next(); // Skip cellsize
    let nodata_value: f32 = lines.next().unwrap()?.split_whitespace().last().unwrap().parse()?;

    // Read the DEM data into a vector.
    let mut data = Vec::with_capacity(ncols * nrows);
    for line in lines {
        for val in line?.split_whitespace() {
            let v: f32 = val.parse().unwrap_or(nodata_value);
            data.push(v);
        }
    }

    Ok((data, ncols, nrows))
}

/// Converts the DEM data to a grayscale image.
fn dem_to_grayscale(dem: &[f32]) -> anyhow::Result<Vec<u8>> {
    let nodata_value = -99999.0;

    // Filter out the no-data values and find the min and max elevations.
    let valid: Vec<f32> = dem.iter().copied().filter(|&v| v != nodata_value).collect();
    let min = valid.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = valid.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let scale = 255.0 / (max - min);

    // Convert the DEM values to grayscale values.
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

/// Converts the DEM data to a color image using a color gradient.
fn dem_to_color_image(dem: &[f32], width: usize, height: usize) -> anyhow::Result<Vec<u8>> {
    let nodata = -99999.0;

    // Filter out the no-data values and find the min and max elevations.
    let valid: Vec<f32> = dem.iter().copied().filter(|&v| v != nodata).collect();
    let min = valid.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = valid.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    // Create a color gradient.
    let grad = colorgrad::turbo();
    let mut rgb_image = Vec::with_capacity(width * height * 3);

    // Convert the DEM values to RGB values using the color gradient.
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

/// Generates a hillshade image from the DEM data.
fn generate_hillshade(dem: &[f32], width: usize, height: usize) -> Vec<u8> {
    let mut image = vec![0u8; width * height];
    let scale = 1.0;
    let azimuth = 315.0_f32.to_radians();
    let altitude = 45.0_f32.to_radians();
    let nodata = -99999.0;

    // Iterate over each pixel in the DEM data to calculate the hillshade.
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let center_idx = y * width + x;

            // Helper function to get the DEM value at a specific offset.
            let get = |dx: isize, dy: isize| {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                    return nodata;
                }
                let i = ny as usize * width + nx as usize;
                dem[i]
            };

            // Calculate the slope and aspect of the terrain.
            let dzdx = ((get(1, -1) + 2.0 * get(1, 0) + get(1, 1)) -
                        (get(-1, -1) + 2.0 * get(-1, 0) + get(-1, 1))) / (8.0 * scale);
            let dzdy = ((get(-1, 1) + 2.0 * get(0, 1) + get(1, 1)) -
                        (get(-1, -1) + 2.0 * get(0, -1) + get(1, -1))) / (8.0 * scale);

            // Skip no-data values.
            if dem[center_idx] == nodata {
                image[center_idx] = 0;
                continue;
            }

            let slope = (dzdx.powi(2) + dzdy.powi(2)).sqrt();
            let aspect = dzdy.atan2(-dzdx);

            // Calculate the hillshade value.
            let shade = (altitude.sin() * (1.0 - slope.atan()).cos() +
                         altitude.cos() * (1.0 - slope.atan()).sin() * (azimuth - aspect).cos())
                         .max(0.0);

            image[center_idx] = (shade * 255.0) as u8;
        }
    }

    image
}

/// Blends a color image with a hillshade image.
fn blend_with_hillshade(rgb: &[u8], shade: &[u8]) -> Vec<u8> {
    // Iterate over each pixel in the color image and blend it with the hillshade.
    rgb.chunks(3)
        .zip(shade.iter())
        .flat_map(|(color, &s)| {
            let factor = s as f32 / 255.0;
            color.iter().map(|&c| (c as f32 * factor) as u8).collect::<Vec<u8>>()
        })
        .collect()
}
