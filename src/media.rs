/// media.rs - Gestione caricamento immagini e video, estrazione frame con FFmpeg
/// e codifica delle anteprime per la UI.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use image::{DynamicImage, ImageFormat, RgbImage};

pub fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let e = ext.to_lowercase();
        matches!(e.as_str(), "mp4" | "mov" | "mkv" | "avi" | "webm" | "m4v" | "flv")
    } else {
        false
    }
}

// Estrae un frame da video utilizzando FFmpeg
pub fn extract_frame_from_video(video_path: &Path) -> Result<RgbImage, String> {
    // Prova a trovare ffmpeg (supporto macOS e Windows)
    let ffmpeg_cmd = if cfg!(target_os = "windows") {
        if Path::new("ffmpeg.exe").exists() {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        }
    } else if Path::new("/opt/homebrew/bin/ffmpeg").exists() {
        "/opt/homebrew/bin/ffmpeg"
    } else if Path::new("/usr/local/bin/ffmpeg").exists() {
        "/usr/local/bin/ffmpeg"
    } else {
        "ffmpeg"
    };

    let tmp_dir = std::env::temp_dir();
    let tmp_output = tmp_dir.join(format!("lutmatch_frame_{}.jpg", std::process::id()));

    // Estrae un fotogramma al 30% della durata (o a 2 secondi per sicurezza)
    let status = Command::new(ffmpeg_cmd)
        .arg("-y")
        .arg("-ss")
        .arg("00:00:02.000")
        .arg("-i")
        .arg(video_path)
        .arg("-vframes")
        .arg("1")
        .arg("-q:v")
        .arg("2")
        .arg(&tmp_output)
        .output();

    let output_result = match status {
        Ok(out) if out.status.success() && tmp_output.exists() => {
            image::open(&tmp_output)
        }
        _ => {
            // Fallback al secondo 0 (primo frame)
            let _ = Command::new(ffmpeg_cmd)
                .arg("-y")
                .arg("-ss")
                .arg("00:00:00.000")
                .arg("-i")
                .arg(video_path)
                .arg("-vframes")
                .arg("1")
                .arg("-q:v")
                .arg("2")
                .arg(&tmp_output)
                .output();
            image::open(&tmp_output)
        }
    };

    let res = match output_result {
        Ok(dyn_img) => Ok(resize_if_needed(dyn_img.to_rgb8(), 1280)),
        Err(e) => Err(format!("Errore estrazione frame da video: {}", e)),
    };

    if tmp_output.exists() {
        let _ = fs::remove_file(tmp_output);
    }

    res
}

pub fn load_media_file(path: &Path) -> Result<RgbImage, String> {
    if is_video_file(path) {
        extract_frame_from_video(path)
    } else {
        match image::open(path) {
            Ok(img) => Ok(resize_if_needed(img.to_rgb8(), 1280)),
            Err(e) => Err(format!("Errore caricamento immagine: {}", e)),
        }
    }
}

// Ridimensiona se supera una dimensione massima per mantenere l'UI ultraveloce
fn resize_if_needed(img: RgbImage, max_dim: u32) -> RgbImage {
    let (w, h) = (img.width(), img.height());
    if w.max(h) > max_dim {
        let scale = max_dim as f32 / (w.max(h) as f32);
        let new_w = (w as f32 * scale).round() as u32;
        let new_h = (h as f32 * scale).round() as u32;
        image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Triangle)
    } else {
        img
    }
}

// Converte un'immagine RGB in stringa data URL JPEG base64 per il rendering immediato nella UI
pub fn rgb_to_base64_data_url(img: &RgbImage) -> String {
    let mut buffer = Cursor::new(Vec::new());
    let dyn_img = DynamicImage::ImageRgb8(img.clone());
    let _ = dyn_img.write_to(&mut buffer, ImageFormat::Jpeg);
    let encoded = BASE64.encode(buffer.into_inner());
    format!("data:image/jpeg;base64,{}", encoded)
}

// Trova la cartella LUT di DaVinci Resolve su macOS e Windows
pub fn get_davinci_lut_dirs() -> Vec<PathBuf> {
    let mut list = Vec::new();
    
    // Supporto Windows
    #[cfg(target_os = "windows")]
    {
        if let Ok(progdata) = std::env::var("ProgramData") {
            list.push(PathBuf::from(progdata).join("Blackmagic Design/DaVinci Resolve/Support/LUT"));
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            list.push(PathBuf::from(appdata).join("Blackmagic Design/DaVinci Resolve/Support/LUT"));
        }
        list.push(PathBuf::from(r"C:\ProgramData\Blackmagic Design\DaVinci Resolve\Support\LUT"));
    }

    // Supporto macOS / Unix
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs_home() {
            list.push(home.join("Library/Application Support/Blackmagic Design/DaVinci Resolve/LUT"));
        }
        list.push(PathBuf::from("/Library/Application Support/Blackmagic Design/DaVinci Resolve/LUT"));
    }

    list
}

#[allow(dead_code)]
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

// Salva la stringa CUBE direttamente nella directory LUT di DaVinci
pub fn save_to_davinci(cube_data: &str, filename: &str, subfolder: &str) -> Result<PathBuf, String> {
    let clean_name = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ')
        .collect::<String>()
        .trim()
        .to_string();

    let clean_name = if clean_name.is_empty() {
        "Auto_Match_Look".to_string()
    } else {
        clean_name
    };

    let fname = format!("{}.cube", clean_name);
    let dirs = get_davinci_lut_dirs();

    for dir in dirs {
        let target_dir = dir.join(subfolder);
        if fs::create_dir_all(&target_dir).is_ok() {
            let file_path = target_dir.join(&fname);
            if fs::write(&file_path, cube_data).is_ok() {
                return Ok(file_path);
            }
        }
    }

    Err("Impossibile salvare nella cartella LUT di DaVinci Resolve. Verifica i permessi.".into())
}
