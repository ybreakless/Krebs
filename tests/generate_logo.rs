use std::fs;
use std::path::Path;

/// The definitive Krebs Studio logo. The test renders this vector into every
/// raster asset the app ships (`logo.png`, `krebs_icon.png`, `krebs_icon.ico`)
/// and round-trips it back to `assets/logo.svg`, so the included file is the
/// single source of truth - edit the SVG, run this test, commit all artifacts.
const CRAB_SVG: &str = include_str!("../assets/logo.svg");

#[test]
fn generate_minimalist_crab_logo() {
    let opt = usvg::Options::default();
    let rtree = usvg::Tree::from_str(CRAB_SVG, &opt).expect("valid SVG");

    let pixmap_size = rtree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&rtree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let png_data = pixmap.encode_png().expect("encode png");
    fs::write(Path::new("assets/logo.png"), &png_data).expect("write assets/logo.png");
    fs::write(Path::new("krebs_icon.png"), &png_data).expect("write krebs_icon.png");

    // Also write standard Windows ICO embedding the PNG
    let mut ico = Vec::new();
    ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]); // ICO header: 1 image
    let width_byte = 0u8; // 0 means 256px
    let height_byte = 0u8;
    let colors = 0u8;
    let reserved = 0u8;
    let planes = 1u16;
    let bpp = 32u16;
    let size = png_data.len() as u32;
    let offset = 22u32; // 6 + 16

    ico.push(width_byte);
    ico.push(height_byte);
    ico.push(colors);
    ico.push(reserved);
    ico.extend_from_slice(&planes.to_le_bytes());
    ico.extend_from_slice(&bpp.to_le_bytes());
    ico.extend_from_slice(&size.to_le_bytes());
    ico.extend_from_slice(&offset.to_le_bytes());
    ico.extend_from_slice(&png_data);
    fs::write(Path::new("krebs_icon.ico"), &ico).expect("write krebs_icon.ico");

    // Also write SVG for web and vector usage
    fs::write(Path::new("assets/logo.svg"), CRAB_SVG).expect("write assets/logo.svg");

    println!("Successfully generated minimalist geometric blue crab logo!");
}
