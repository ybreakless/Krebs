use std::fs;
use std::path::Path;

const CRAB_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="512" height="512">
  <defs>
    <radialGradient id="bgGlow" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#1c1f28" stop-opacity="1"/>
      <stop offset="100%" stop-color="#0a0b0e" stop-opacity="1"/>
    </radialGradient>
    <linearGradient id="crabGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#ff6452"/>
      <stop offset="100%" stop-color="#cf2d2d"/>
    </linearGradient>
    <linearGradient id="clawGrad" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#ff7b69"/>
      <stop offset="100%" stop-color="#ba2323"/>
    </linearGradient>
    <linearGradient id="facetGrad" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="#ff8573" stop-opacity="0.35"/>
      <stop offset="100%" stop-color="#941616" stop-opacity="0.4"/>
    </linearGradient>
  </defs>

  <!-- Dark Rounded Container -->
  <rect width="512" height="512" rx="112" fill="url(#bgGlow)"/>
  <rect x="18" y="18" width="476" height="476" rx="96" fill="none" stroke="rgba(255,255,255,0.08)" stroke-width="2.5"/>

  <!-- Minimalist Geometric Crab Mascot -->
  <g transform="translate(256, 268)">
    <!-- 6 Walking Legs (3 Left, 3 Right) -->
    <path d="M-105 -22 L-170 -60 L-198 -30" fill="none" stroke="#d63031" stroke-width="14" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M-112 16 L-184 10 L-208 52" fill="none" stroke="#d63031" stroke-width="14" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M-102 52 L-168 88 L-188 138" fill="none" stroke="#d63031" stroke-width="14" stroke-linecap="round" stroke-linejoin="round"/>

    <path d="M105 -22 L170 -60 L198 -30" fill="none" stroke="#d63031" stroke-width="14" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M112 16 L184 10 L208 52" fill="none" stroke="#d63031" stroke-width="14" stroke-linecap="round" stroke-linejoin="round"/>
    <path d="M102 52 L168 88 L188 138" fill="none" stroke="#d63031" stroke-width="14" stroke-linecap="round" stroke-linejoin="round"/>

    <!-- Left Arm & Claw -->
    <path d="M-72 -50 Q-115 -115 -148 -132" fill="none" stroke="#cf2d2d" stroke-width="18" stroke-linecap="round"/>
    <!-- Left Pincer Cutout -->
    <path d="M-138 -122 C-180 -182 -118 -212 -98 -182 C-102 -156 -122 -146 -138 -122 Z" fill="url(#clawGrad)"/>
    <path d="M-146 -118 C-202 -142 -182 -202 -148 -186 C-144 -166 -144 -142 -146 -118 Z" fill="url(#clawGrad)"/>

    <!-- Right Arm & Claw -->
    <path d="M72 -50 Q115 -115 148 -132" fill="none" stroke="#cf2d2d" stroke-width="18" stroke-linecap="round"/>
    <!-- Right Pincer Cutout -->
    <path d="M138 -122 C180 -182 118 -212 98 -182 C102 -156 122 -146 138 -122 Z" fill="url(#clawGrad)"/>
    <path d="M146 -118 C202 -142 182 -202 148 -186 C144 -166 144 -142 146 -118 Z" fill="url(#clawGrad)"/>

    <!-- Geometric Carapace (Shell) -->
    <path d="M-118 12 C-124 -62 -64 -88 0 -88 C64 -88 124 -62 118 12 C114 80 58 96 0 96 C-58 96 -114 80 -118 12 Z" fill="url(#crabGrad)"/>
    
    <!-- Facet Accents -->
    <path d="M-78 -10 C-38 -42 38 -42 78 -10 C48 46 -48 46 -78 -10 Z" fill="url(#facetGrad)"/>

    <!-- Eyes (Stalks + Pupils) -->
    <circle cx="-38" cy="-86" r="14" fill="#ffffff"/>
    <circle cx="-38" cy="-88" r="7.5" fill="#111216"/>
    <circle cx="38" cy="-86" r="14" fill="#ffffff"/>
    <circle cx="38" cy="-88" r="7.5" fill="#111216"/>

    <!-- Drafting Nib / Pen Tip (Minimalist Writing Studio Symbol) -->
    <path d="M0 -30 L16 12 L0 42 L-16 12 Z" fill="#ffffff" opacity="0.92"/>
    <line x1="0" y1="-26" x2="0" y2="28" stroke="#111216" stroke-width="2.5" stroke-linecap="round"/>
    <circle cx="0" cy="8" r="3" fill="#111216"/>
  </g>
</svg>
"##;

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

    println!("Successfully generated minimalist geometric crab logo!");
}
