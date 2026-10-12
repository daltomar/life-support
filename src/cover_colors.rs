use palette::{FromColor, IntoColor, Lab, Srgb};
use std::path::Path;

/// Page background colour (CSS `--panel: #1A1917`).
const BG: [u8; 3] = [0x1A, 0x19, 0x17];

/// A pixel is considered chromatic if its Lab chroma exceeds this value.
const MIN_CHROMA: f32 = 5.0;
/// If fewer than this fraction of pixels are chromatic, return None.
const MIN_CHROMATIC_FRACTION: f32 = 0.02;

#[derive(Clone, Debug, PartialEq)]
pub struct CoverPalette {
    pub accent: String,       // "#rrggbb" — headings, play button
    pub accent_muted: String, // "#rrggbb" — borders, dividers
}

/// Extract a contrast-adjusted accent colour from the image at `path`.
/// Returns `Ok(None)` for near-greyscale images.
/// Returns `Err` on I/O or decode failure.
pub fn extract_palette(path: &Path) -> Result<Option<CoverPalette>, String> {
    let img = image::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .thumbnail(64, 64)
        .into_rgb8();

    let pixels: Vec<Lab> = img
        .pixels()
        .map(|p| {
            let srgb: Srgb<f32> = Srgb::new(
                p[0] as f32 / 255.0,
                p[1] as f32 / 255.0,
                p[2] as f32 / 255.0,
            );
            srgb.into_color()
        })
        .collect();

    // Collect chromatic pixels — those with meaningful saturation.
    // Dark-saturated colors (L < 15 but high chroma) are valid and must not
    // be excluded here; contrast_adjust will lighten them for readability.
    let chromatic: Vec<Lab> = pixels
        .iter()
        .copied()
        .filter(|lab| chroma(*lab) > MIN_CHROMA)
        .collect();

    if (chromatic.len() as f32) < (pixels.len() as f32 * MIN_CHROMATIC_FRACTION) {
        return Ok(None);
    }

    // Bucket chromatic pixels by hue into 36 × 10° bins, weighted by chroma.
    let mut bins = [0f32; 36];
    let mut bin_sum: [Lab; 36] = [Lab::new(0.0f32, 0.0, 0.0); 36];
    let mut bin_count = [0u32; 36];

    for lab in &chromatic {
        let c = chroma(*lab);
        let hue_deg = lab.b.atan2(lab.a).to_degrees().rem_euclid(360.0);
        let idx = (hue_deg / 10.0) as usize % 36;
        bins[idx] += c;
        bin_sum[idx].l += lab.l;
        bin_sum[idx].a += lab.a;
        bin_sum[idx].b += lab.b;
        bin_count[idx] += 1;
    }

    let best = bins
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);

    let count = bin_count[best].max(1) as f32;
    let raw_lab = Lab::new(
        bin_sum[best].l / count,
        bin_sum[best].a / count,
        bin_sum[best].b / count,
    );

    let accent = contrast_adjust(&lab_to_hex(raw_lab));

    // Muted: lower lightness, reduce chroma.
    let muted_lab = Lab::new(
        (raw_lab.l - 15.0).max(0.0),
        raw_lab.a * 0.7,
        raw_lab.b * 0.7,
    );
    let accent_muted = lab_to_hex(muted_lab);

    Ok(Some(CoverPalette { accent, accent_muted }))
}

/// Lighten `hex` in Lab space until WCAG AA contrast (4.5:1) against `BG`.
/// Returns a valid lowercase `#rrggbb` string.
pub(crate) fn contrast_adjust(hex: &str) -> String {
    let Some([r, g, b]) = parse_hex(hex) else {
        return "#ececec".to_string();
    };
    let bg_lum = relative_luminance(BG[0], BG[1], BG[2]);
    let mut lab: Lab = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        .into_color();

    for _ in 0..20 {
        let srgb: Srgb<f32> = Lab::from_color(lab).into_color();
        let (cr, cg, cb) = clamp_srgb(srgb);
        let fg_lum = relative_luminance(cr, cg, cb);
        let (light, dark) = if fg_lum > bg_lum { (fg_lum, bg_lum) } else { (bg_lum, fg_lum) };
        if (light + 0.05) / (dark + 0.05) >= 4.5 {
            return format!("#{:02x}{:02x}{:02x}", cr, cg, cb);
        }
        lab.l = (lab.l + 5.0).min(95.0);
    }

    let srgb: Srgb<f32> = Lab::from_color(lab).into_color();
    let (cr, cg, cb) = clamp_srgb(srgb);
    format!("#{:02x}{:02x}{:02x}", cr, cg, cb)
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn chroma(lab: Lab) -> f32 {
    (lab.a * lab.a + lab.b * lab.b).sqrt()
}

fn lab_to_hex(lab: Lab) -> String {
    let srgb: Srgb<f32> = Lab::from_color(lab).into_color();
    let (r, g, b) = clamp_srgb(srgb);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn clamp_srgb(srgb: Srgb<f32>) -> (u8, u8, u8) {
    let clamp = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    (clamp(srgb.red), clamp(srgb.green), clamp(srgb.blue))
}

fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let s = hex.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r, g, b])
}

/// WCAG 2.1 relative luminance from 8-bit sRGB components.
pub(crate) fn relative_luminance(r: u8, g: u8, b: u8) -> f32 {
    fn lin(c: u8) -> f32 {
        let v = c as f32 / 255.0;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    }
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn contrast_ratio(hex: &str, bg: [u8; 3]) -> f32 {
        let [r, g, b] = parse_hex(hex).unwrap();
        let fg = relative_luminance(r, g, b);
        let bk = relative_luminance(bg[0], bg[1], bg[2]);
        let (light, dark) = if fg > bk { (fg, bk) } else { (bk, fg) };
        (light + 0.05) / (dark + 0.05)
    }

    #[test]
    fn already_contrasting_unchanged() {
        // #FFFFFF has ~21:1 on #1A1917 — must be returned as-is.
        let result = contrast_adjust("#FFFFFF");
        assert_eq!(result, "#ffffff");
    }

    #[test]
    fn dark_colour_lightened() {
        let result = contrast_adjust("#1a0000");
        let [r_in, g_in, b_in] = parse_hex("#1a0000").unwrap();
        let [r_out, g_out, b_out] = parse_hex(&result).unwrap();
        let lum_in  = relative_luminance(r_in,  g_in,  b_in);
        let lum_out = relative_luminance(r_out, g_out, b_out);
        assert!(lum_out > lum_in, "adjusted colour should be lighter");
    }

    #[test]
    fn adjusted_meets_wcag_aa() {
        for hex in &["#3a1a1a", "#001a00", "#00001a", "#800000"] {
            let adjusted = contrast_adjust(hex);
            let ratio = contrast_ratio(&adjusted, BG);
            assert!(
                ratio >= 4.5,
                "contrast ratio {ratio:.2} < 4.5:1 for input {hex} → {adjusted}"
            );
        }
    }

    #[test]
    fn hex_case_normalised() {
        assert_eq!(contrast_adjust("#FF4500"), contrast_adjust("#ff4500"));
    }

    #[test]
    fn invalid_hex_no_panic() {
        let result = contrast_adjust("not-a-color");
        assert!(result.starts_with('#'), "result should start with #");
        assert_eq!(result.len(), 7, "result should be 7 chars");
    }

    #[test]
    fn missing_file_returns_err() {
        let result = extract_palette(Path::new("/nonexistent/cover.webp"));
        assert!(result.is_err());
    }
}
