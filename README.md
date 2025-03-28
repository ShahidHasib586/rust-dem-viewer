# Rust DEM Viewer

A terminal-based Digital Elevation Model (DEM) viewer that supports grayscale, color gradient, hillshade, and color+hillshade rendering of `.asc` files.

## 👨‍💻 Developers

- Shahid Ahamed Hasib
- Mohamed Magdy Atta

## ✅ Features

- Opens and parses `.asc` DEM files.
- Displays DEM in:
  - Grayscale (Q1)
  - Color (Turbo colormap) (Q2)
  - Hillshade (simulated sunlight shading) (Q2)
  - Color + Hillshade (enhanced terrain visualization) (Q2)

## 🚀 Requirements

- Rust (recommended: `rustup` + stable toolchain)
- Git
- Internet (for fetching crates on first build)

## 📦 Installation

```bash
git clone https://github.com/shahidhasib586/rust-dem-viewer.git
cd rust-dem-viewer
cargo build --release
```

## ▶️ Running the Program

### General Syntax

```bash
cargo run --release -- <path-to-asc-file> --mode <grayscale | color | hillshade | color+hillshade>
```

### Example

```bash
cargo run --release -- "/home/shahidhasib586/Downloads/0925_6225/LITTO3D_FRA_0925_6224_...asc" --mode color+hillshade
```

### Available Modes

- `grayscale`: Renders elevation in grayscale.
- `color`: Maps elevation to the Turbo colormap.
- `hillshade`: Applies terrain shading from the northwest light source.
- `color+hillshade`: Blends colormap and hillshade for an enhanced view.

## 📄 Notes

- The DEM data must be in the ASCII Grid (`.asc`) format.
- Nodata values (typically `-99999.0`) are displayed in black.
- This program uses the following crates:
  - `show-image` for rendering
  - `colorgrad` for gradients
  - `clap` for argument parsing
  - `anyhow` for error handling

## 🧪 Testing

All features have been manually tested on `.asc` files from the LITTO3D dataset.

## 💡 Additional Features (Q3)

- [Add others.]

---

Made with ❤️ for DEM visualization and terrain exploration.

