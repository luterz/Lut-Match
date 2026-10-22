//! main.rs - LutMatch Studio Desktop App in Rust (Three-Pane Apple Liquid Layout)

mod color;
mod media;

use std::sync::{Arc, Mutex};
use std::thread;

use color::{apply_look_to_image, compute_image_stats, generate_cube_string, ColorStats};
use image::RgbImage;
use media::{load_media_file, rgb_to_base64_data_url, save_to_davinci};
use serde::{Deserialize, Serialize};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use tiny_http::{Header, Response, Server};
use wry::WebViewBuilder;

#[derive(Default)]
struct AppState {
    src_image: Option<RgbImage>,
    tgt_image: Option<RgbImage>,
    src_stats: Option<ColorStats>,
    tgt_stats: Option<ColorStats>,
    src_name: Option<String>,
    tgt_name: Option<String>,
    last_cube: Option<String>,
}

#[derive(Deserialize)]
struct GenerateReq {
    blend: Option<u32>,
}

#[derive(Deserialize)]
struct SaveDavinciReq {
    name: Option<String>,
    blend: Option<u32>,
}

#[derive(Serialize)]
struct SelectResp {
    status: String,
    filename: String,
    preview: String,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct GenerateResp {
    status: String,
    preview: String,
}

#[derive(Serialize)]
struct SimpleResp {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

const UI_HTML: &str = include_str!("ui.html");

fn main() {
    let state = Arc::new(Mutex::new(AppState::default()));

    // Avvia server HTTP embedded su 127.0.0.1
    let server = Server::http("127.0.0.1:0")
        .or_else(|_| Server::http("127.0.0.1:7865"))
        .expect("Impossibile avviare il server HTTP embedded");
    let server_port = server.server_addr().to_ip().map(|ip| ip.port()).unwrap_or(7865);
    let server_url = format!("http://127.0.0.1:{}/", server_port);

    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let url = request.url().to_string();
            let json_hdr = Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..]).unwrap();
            let html_hdr = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();

            match url.as_str() {
                "/" | "/index.html" => {
                    let resp = Response::from_string(UI_HTML).with_header(html_hdr);
                    let _ = request.respond(resp);
                }
                "/api/select-source" => {
                    let file_opt = rfd::FileDialog::new()
                        .set_title("Seleziona Video o Foto Rec.709")
                        .add_filter("Media (Video e Foto)", &["mp4", "mov", "mkv", "avi", "webm", "png", "jpg", "jpeg", "tiff", "webp"])
                        .pick_file();

                    if let Some(path) = file_opt {
                        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("source").to_string();
                        match load_media_file(&path) {
                            Ok(img) => {
                                let stats = compute_image_stats(&img);
                                let preview = rgb_to_base64_data_url(&img);
                                let (w, h) = (img.width(), img.height());

                                {
                                    let mut s = state_clone.lock().unwrap();
                                    s.src_image = Some(img);
                                    s.src_stats = Some(stats);
                                    s.src_name = Some(filename.clone());
                                }

                                let res_body = serde_json::to_string(&SelectResp {
                                    status: "ok".into(),
                                    filename,
                                    preview,
                                    width: w,
                                    height: h,
                                }).unwrap();
                                let _ = request.respond(Response::from_string(res_body).with_header(json_hdr));
                            }
                            Err(e) => {
                                let err_body = serde_json::to_string(&SimpleResp {
                                    status: "error".into(),
                                    path: None,
                                    filename: None,
                                    error: Some(e),
                                }).unwrap();
                                let _ = request.respond(Response::from_string(err_body).with_header(json_hdr));
                            }
                        }
                    } else {
                        let cancel_body = serde_json::to_string(&SimpleResp {
                            status: "cancel".into(),
                            path: None,
                            filename: None,
                            error: None,
                        }).unwrap();
                        let _ = request.respond(Response::from_string(cancel_body).with_header(json_hdr));
                    }
                }
                "/api/select-target" => {
                    let file_opt = rfd::FileDialog::new()
                        .set_title("Seleziona Riferimento Color (Foto o Video)")
                        .add_filter("Media (Foto e Video)", &["png", "jpg", "jpeg", "tiff", "webp", "mp4", "mov", "mkv", "avi", "webm"])
                        .pick_file();

                    if let Some(path) = file_opt {
                        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("target").to_string();
                        match load_media_file(&path) {
                            Ok(img) => {
                                let stats = compute_image_stats(&img);
                                let preview = rgb_to_base64_data_url(&img);
                                let (w, h) = (img.width(), img.height());

                                {
                                    let mut s = state_clone.lock().unwrap();
                                    s.tgt_image = Some(img);
                                    s.tgt_stats = Some(stats);
                                    s.tgt_name = Some(filename.clone());
                                }

                                let res_body = serde_json::to_string(&SelectResp {
                                    status: "ok".into(),
                                    filename,
                                    preview,
                                    width: w,
                                    height: h,
                                }).unwrap();
                                let _ = request.respond(Response::from_string(res_body).with_header(json_hdr));
                            }
                            Err(e) => {
                                let err_body = serde_json::to_string(&SimpleResp {
                                    status: "error".into(),
                                    path: None,
                                    filename: None,
                                    error: Some(e),
                                }).unwrap();
                                let _ = request.respond(Response::from_string(err_body).with_header(json_hdr));
                            }
                        }
                    } else {
                        let cancel_body = serde_json::to_string(&SimpleResp {
                            status: "cancel".into(),
                            path: None,
                            filename: None,
                            error: None,
                        }).unwrap();
                        let _ = request.respond(Response::from_string(cancel_body).with_header(json_hdr));
                    }
                }
                "/api/generate" => {
                    let mut body_str = String::new();
                    let _ = request.as_reader().read_to_string(&mut body_str);
                    let req_data: GenerateReq = serde_json::from_str(&body_str).unwrap_or(GenerateReq { blend: Some(100) });
                    let blend = (req_data.blend.unwrap_or(100) as f32) / 100.0;

                    let (preview_b64, cube) = {
                        let mut s = state_clone.lock().unwrap();
                        if let (Some(src_img), Some(src_stats), Some(tgt_stats)) = (&s.src_image, &s.src_stats, &s.tgt_stats) {
                            let out_img = apply_look_to_image(src_img, src_stats, tgt_stats, blend);
                            let preview = rgb_to_base64_data_url(&out_img);
                            let title = s.tgt_name.clone().unwrap_or_else(|| "LutMatch_Style".into());
                            let cube = generate_cube_string(src_stats, tgt_stats, 33, blend, &title);
                            s.last_cube = Some(cube.clone());
                            (Some(preview), Some(cube))
                        } else {
                            (None, None)
                        }
                    };

                    if let (Some(p), Some(_)) = (preview_b64, cube) {
                        let res_body = serde_json::to_string(&GenerateResp {
                            status: "ok".into(),
                            preview: p,
                        }).unwrap();
                        let _ = request.respond(Response::from_string(res_body).with_header(json_hdr));
                    } else {
                        let err_body = serde_json::to_string(&SimpleResp {
                            status: "error".into(),
                            path: None,
                            filename: None,
                            error: Some("Carica sia la sorgente che il riferimento colore.".into()),
                        }).unwrap();
                        let _ = request.respond(Response::from_string(err_body).with_header(json_hdr));
                    }
                }
                "/api/save-as" => {
                    let mut body_str = String::new();
                    let _ = request.as_reader().read_to_string(&mut body_str);
                    let req_data: GenerateReq = serde_json::from_str(&body_str).unwrap_or(GenerateReq { blend: Some(100) });
                    let blend = (req_data.blend.unwrap_or(100) as f32) / 100.0;

                    let cube_data = {
                        let mut s = state_clone.lock().unwrap();
                        if s.last_cube.is_none() {
                            if let (Some(src_stats), Some(tgt_stats)) = (&s.src_stats, &s.tgt_stats) {
                                s.last_cube = Some(generate_cube_string(src_stats, tgt_stats, 33, blend, "Custom_Look"));
                            }
                        }
                        s.last_cube.clone()
                    };

                    if let Some(cube) = cube_data {
                        if let Some(save_path) = rfd::FileDialog::new()
                            .set_title("Salva File .cube")
                            .set_file_name("Moody_Look.cube")
                            .add_filter("DaVinci 3D LUT", &["cube"])
                            .save_file()
                        {
                            let _ = std::fs::write(&save_path, cube);
                            let res_body = serde_json::to_string(&SimpleResp {
                                status: "ok".into(),
                                path: Some(save_path.to_string_lossy().to_string()),
                                filename: save_path.file_name().and_then(|f| f.to_str()).map(Into::into),
                                error: None,
                            }).unwrap();
                            let _ = request.respond(Response::from_string(res_body).with_header(json_hdr));
                        } else {
                            let _ = request.respond(Response::from_string(r#"{"status":"cancel"}"#).with_header(json_hdr));
                        }
                    } else {
                        let _ = request.respond(Response::from_string(r#"{"status":"error","error":"Nessuna LUT generata"}"#).with_header(json_hdr));
                    }
                }
                "/api/save-davinci" => {
                    let mut body_str = String::new();
                    let _ = request.as_reader().read_to_string(&mut body_str);
                    let req_data: SaveDavinciReq = serde_json::from_str(&body_str).unwrap_or(SaveDavinciReq {
                        name: Some("Moody_Cinematic".into()),
                        blend: Some(100),
                    });
                    let blend = (req_data.blend.unwrap_or(100) as f32) / 100.0;
                    let name = req_data.name.unwrap_or_else(|| "Moody_Cinematic".into());

                    let cube_data = {
                        let mut s = state_clone.lock().unwrap();
                        if s.last_cube.is_none() {
                            if let (Some(src_stats), Some(tgt_stats)) = (&s.src_stats, &s.tgt_stats) {
                                s.last_cube = Some(generate_cube_string(src_stats, tgt_stats, 33, blend, &name));
                            }
                        }
                        s.last_cube.clone()
                    };

                    if let Some(cube) = cube_data {
                        match save_to_davinci(&cube, &name, "LutMatch") {
                            Ok(saved_path) => {
                                let res_body = serde_json::to_string(&SimpleResp {
                                    status: "ok".into(),
                                    path: Some(saved_path.to_string_lossy().to_string()),
                                    filename: saved_path.file_name().and_then(|f| f.to_str()).map(Into::into),
                                    error: None,
                                }).unwrap();
                                let _ = request.respond(Response::from_string(res_body).with_header(json_hdr));
                            }
                            Err(e) => {
                                let _ = request.respond(Response::from_string(format!(r#"{{"status":"error","error":"{}"}}"#, e)).with_header(json_hdr));
                            }
                        }
                    } else {
                        let _ = request.respond(Response::from_string(r#"{"status":"error","error":"Genera prima la LUT"}"#).with_header(json_hdr));
                    }
                }
                _ => {
                    let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
                }
            }
        }
    });

    // Crea Finestra Desktop Standalone Nativa macOS con Tao + Wry
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("LutMatch Studio")
        .with_inner_size(tao::dpi::LogicalSize::new(1340.0, 860.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(980.0, 700.0))
        .build(&event_loop)
        .expect("Impossibile creare finestra nativa");

    let _webview = WebViewBuilder::new()
        .with_url(&server_url)
        .build(&window)
        .expect("Impossibile inizializzare WebKit");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
