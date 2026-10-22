/// color.rs - Motore di conversione colorimetrica sRGB <-> Lab,
/// Reinhard Color Transfer e generazione di 3D LUT (.cube).

use image::RgbImage;

#[derive(Clone, Copy, Debug)]
pub struct ColorStats {
    pub mean_l: f32,
    pub mean_a: f32,
    pub mean_b: f32,
    pub std_l: f32,
    pub std_a: f32,
    pub std_b: f32,
}

#[inline]
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// Conversione da sRGB [0, 1] a CIE Lab
#[inline]
pub fn rgb_to_lab(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let lr = srgb_to_linear(r);
    let lg = srgb_to_linear(g);
    let lb = srgb_to_linear(b);

    // D65 Reference White
    let x = (0.4124564 * lr + 0.3575761 * lg + 0.1804375 * lb) / 0.95047;
    let y = (0.2126729 * lr + 0.7151522 * lg + 0.0721750 * lb) / 1.00000;
    let z = (0.0193339 * lr + 0.1191920 * lg + 0.9503041 * lb) / 1.08883;

    #[inline]
    fn f(t: f32) -> f32 {
        const DELTA: f32 = 6.0 / 29.0;
        if t > DELTA * DELTA * DELTA {
            t.cbrt()
        } else {
            t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
        }
    }

    let fx = f(x);
    let fy = f(y);
    let fz = f(z);

    let l = (116.0 * fy - 16.0).clamp(0.0, 100.0);
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);

    (l, a, b)
}

// Conversione inversa da CIE Lab a sRGB [0, 1]
#[inline]
pub fn lab_to_rgb(l: f32, a: f32, b: f32) -> (f32, f32, f32) {
    #[inline]
    fn f_inv(t: f32) -> f32 {
        const DELTA: f32 = 6.0 / 29.0;
        if t > DELTA {
            t * t * t
        } else {
            3.0 * DELTA * DELTA * (t - 4.0 / 29.0)
        }
    }

    let fy = (l + 16.0) / 116.0;
    let fx = fy + a / 500.0;
    let fz = fy - b / 200.0;

    let x = f_inv(fx) * 0.95047;
    let y = f_inv(fy) * 1.00000;
    let z = f_inv(fz) * 1.08883;

    let lr = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    let lg = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
    let lb = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

    (
        linear_to_srgb(lr).clamp(0.0, 1.0),
        linear_to_srgb(lg).clamp(0.0, 1.0),
        linear_to_srgb(lb).clamp(0.0, 1.0),
    )
}

// Calcola le statistiche Lab su un'immagine RGB
pub fn compute_image_stats(img: &RgbImage) -> ColorStats {
    let mut sum_l = 0.0f64;
    let mut sum_a = 0.0f64;
    let mut sum_b = 0.0f64;
    let count = (img.width() * img.height()) as f64;

    for pixel in img.pixels() {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;
        let (l, a, b) = rgb_to_lab(r, g, b);
        sum_l += l as f64;
        sum_a += a as f64;
        sum_b += b as f64;
    }

    let mean_l = (sum_l / count) as f32;
    let mean_a = (sum_a / count) as f32;
    let mean_b = (sum_b / count) as f32;

    let mut var_l = 0.0f64;
    let mut var_a = 0.0f64;
    let mut var_b = 0.0f64;

    for pixel in img.pixels() {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;
        let (l, a, b) = rgb_to_lab(r, g, b);
        let dl = l - mean_l;
        let da = a - mean_a;
        let db = b - mean_b;
        var_l += (dl * dl) as f64;
        var_a += (da * da) as f64;
        var_b += (db * db) as f64;
    }

    let std_l = ((var_l / count).sqrt() as f32).max(1e-4);
    let std_a = ((var_a / count).sqrt() as f32).max(1e-4);
    let std_b = ((var_b / count).sqrt() as f32).max(1e-4);

    ColorStats {
        mean_l,
        mean_a,
        mean_b,
        std_l,
        std_a,
        std_b,
    }
}

// Applica il trasferimento colore a un singolo valore RGB
#[inline]
pub fn transfer_rgb_pixel(
    r: f32,
    g: f32,
    b: f32,
    src_stats: &ColorStats,
    tgt_stats: &ColorStats,
    blend: f32,
) -> (f32, f32, f32) {
    let (l, a, b_val) = rgb_to_lab(r, g, b);

    // Limitazione rapporto deviazioni standard per prevenire banding o eccessi
    let ratio_l = (tgt_stats.std_l / src_stats.std_l).clamp(0.4, 2.5);
    let ratio_a = (tgt_stats.std_a / src_stats.std_a).clamp(0.4, 2.5);
    let ratio_b = (tgt_stats.std_b / src_stats.std_b).clamp(0.4, 2.5);

    let mapped_l = (l - src_stats.mean_l) * ratio_l + tgt_stats.mean_l;
    let mapped_a = (a - src_stats.mean_a) * ratio_a + tgt_stats.mean_a;
    let mapped_b = (b_val - src_stats.mean_b) * ratio_b + tgt_stats.mean_b;

    let final_l = ((1.0 - blend) * l + blend * mapped_l).clamp(0.0, 100.0);
    let final_a = ((1.0 - blend) * a + blend * mapped_a).clamp(-128.0, 127.0);
    let final_b = ((1.0 - blend) * b_val + blend * mapped_b).clamp(-128.0, 127.0);

    lab_to_rgb(final_l, final_a, final_b)
}

// Applica il trasferimento colore all'intera immagine sorgente per la terza finestra di preview
pub fn apply_look_to_image(
    src_img: &RgbImage,
    src_stats: &ColorStats,
    tgt_stats: &ColorStats,
    blend: f32,
) -> RgbImage {
    let mut out = RgbImage::new(src_img.width(), src_img.height());
    for (src_pix, out_pix) in src_img.pixels().zip(out.pixels_mut()) {
        let r = src_pix[0] as f32 / 255.0;
        let g = src_pix[1] as f32 / 255.0;
        let b = src_pix[2] as f32 / 255.0;
        let (out_r, out_g, out_b) = transfer_rgb_pixel(r, g, b, src_stats, tgt_stats, blend);
        out_pix[0] = (out_r * 255.0).round() as u8;
        out_pix[1] = (out_g * 255.0).round() as u8;
        out_pix[2] = (out_b * 255.0).round() as u8;
    }
    out
}

// Genera il file .cube standard per DaVinci Resolve
pub fn generate_cube_string(
    src_stats: &ColorStats,
    tgt_stats: &ColorStats,
    lut_size: usize,
    blend: f32,
    title: &str,
) -> String {
    let mut out = String::with_capacity(lut_size * lut_size * lut_size * 32);
    out.push_str(&format!("TITLE \"{}\"\n", title));
    out.push_str(&format!("LUT_3D_SIZE {}\n", lut_size));
    out.push_str("DOMAIN_MIN 0.0 0.0 0.0\n");
    out.push_str("DOMAIN_MAX 1.0 1.0 1.0\n\n");

    let step = 1.0 / ((lut_size - 1) as f32);

    // Ordine standard Adobe / DaVinci Resolve:
    // Ciclo esterno: Blue, Ciclo medio: Green, Ciclo interno: Red
    for b_idx in 0..lut_size {
        let in_b = (b_idx as f32) * step;
        for g_idx in 0..lut_size {
            let in_g = (g_idx as f32) * step;
            for r_idx in 0..lut_size {
                let in_r = (r_idx as f32) * step;
                let (out_r, out_g, out_b) = transfer_rgb_pixel(in_r, in_g, in_b, src_stats, tgt_stats, blend);
                out.push_str(&format!("{:.6} {:.6} {:.6}\n", out_r, out_g, out_b));
            }
        }
    }

    out
}
