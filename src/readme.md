# DEM Viewer Project

This project is a simple yet powerful **Digital Elevation Model (DEM)** viewer that supports various display modes for `.asc` raster files, including grayscale, color gradient, hillshading, and a blended color+hillshade visualization.

---

## 👨‍💻 Developers

- Shahid Ahamed Hasib  
- Mohamed Magdy Atta

---

## 📄 Features Implemented

### ✅ Q1: Grayscale DEM Display
- Reads an `.asc` file and visualizes the DEM in grayscale.
- Nodata values (`-99999.0`) are rendered black.

### ✅ Q2: Color Gradient Visualization
- Elevation values are mapped to a **turbo colormap** using the `colorgrad` crate.
- Nodata areas are shown as black.

### ✅ Q2 (part 2): Hillshading Algorithm
- Implements the **hillshade algorithm** explained by ESRI [here](https://pro.arcgis.com/en/pro-app/latest/tool-reference/3d-analyst/how-hillshade-works.htm).
- Produces shaded relief images to simulate terrain lighting from the northwest.

### ✅ Q2: Color + Hillshade Blending
- Blends hillshade shading with the turbo color image to improve visual clarity.

---

## 🛠️ Usage

### ✅ Running the Program

```bash
cargo run -- <path-to-asc-file> --mode <grayscale | color | hillshade | color+hillshade>
