#![windows_subsystem = "windows"]
mod wallpaper;

use std::error::Error;
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use wry::WebViewBuilder;
use windows::Win32::UI::WindowsAndMessaging::SetParent;
use windows::Win32::Foundation::HWND;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let worker_w = wallpaper::get_worker_w().ok_or("Could not find WorkerW window")?;

    let monitor = event_loop.primary_monitor().ok_or("No monitor found")?;
    let size = monitor.size();

    #[allow(deprecated)]
    let window = winit::window::WindowBuilder::new()
        .with_title("Video Wallpaper")
        .with_decorations(false)
        .with_inner_size(size)
        .build(&event_loop)?;

    window.set_outer_position(PhysicalPosition::new(0, 0));

    let window_handle = window.window_handle()?.as_raw();
    let winit_hwnd = match window_handle {
        RawWindowHandle::Win32(handle) => handle.hwnd.get(),
        _ => return Err("Unsupported platform format".into()),
    } as isize;

    unsafe {
        let _ = SetParent(HWND(winit_hwnd as *mut _), Some(worker_w));
    }

    let current_dir = env::current_dir()?;
    let mut video_path = current_dir.join("video.mp4");
    
    if !video_path.exists() {
        let alt_path = current_dir.join("video");
        if alt_path.exists() {
            video_path = alt_path;
        } else {
            if let Ok(entries) = std::fs::read_dir(&current_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().is_some_and(|ext| ext == "mp4") {
                        video_path = entry.path();
                        break;
                    }
                }
            }
        }
    }
    
    if !video_path.exists() {
        return Err(format!("Could not find video file (checked video.mp4, video, and other .mp4 files in {:?})", current_dir).into());
    }

    let html_content = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body {{ margin: 0; padding: 0; overflow: hidden; background-color: black; }}
                video {{ width: 100vw; height: 100vh; object-fit: cover; position: absolute; top: 0; left: 0; z-index: 1; }}
            </style>
        </head>
        <body>
            <video autoplay loop muted playsinline id="v">
                <source src="/video.mp4" type="video/mp4">
            </video>
            <script>
                const v = document.getElementById('v');
                v.onplay = () => {{
                    document.body.style.backgroundColor = "transparent";
                }};
            </script>
        </body>
        </html>
        "#
    );

    let video_path_clone = video_path.clone();
    let _webview = WebViewBuilder::new()
        .with_url("asset://localhost/")
        .with_custom_protocol("asset".into(), move |_url_str, request| {
            let path = request.uri().path();
            
            if path == "/" || path == "/index.html" || path == "" {
                http::Response::builder()
                    .header("Content-Type", "text/html")
                    .body(std::borrow::Cow::from(html_content.clone().into_bytes()))
                    .unwrap()
            } else if path == "/video.mp4" {
                match std::fs::read(&video_path_clone) {
                    Ok(content) => {
                        http::Response::builder()
                            .header("Content-Type", "video/mp4")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(std::borrow::Cow::from(content))
                            .unwrap()
                    }
                    Err(_) => {
                        http::Response::builder()
                            .status(500)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(std::borrow::Cow::from(Vec::new()))
                            .unwrap()
                    }
                }
            } else {
                http::Response::builder()
                    .status(404)
                    .body(std::borrow::Cow::from(Vec::new()))
                    .unwrap()
            }
        })
        .with_background_color((0, 0, 0, 0))
        .build(&window)?;

    window.set_visible(true);

    #[allow(deprecated)]
    event_loop.run(move |event, elwt| {
        let _ = &_webview;
        elwt.set_control_flow(ControlFlow::Wait);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => elwt.exit(),
            _ => (),
        }
    })?;

    Ok(())
}
