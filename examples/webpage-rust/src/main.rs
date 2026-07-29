mod app;
mod site;
mod sites;

use std::process;

use winit::event_loop::EventLoop;

fn main() {
    let mut site_name: Option<String> = None;
    let mut screenshot_path = None;
    let mut exit_after = false;
    let mut delay_secs = 5.0;
    let mut width: u32 = 1280;
    let mut height: u32 = 900;

    let mut args = std::env::args().peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--site" => {
                site_name = args.next();
            }
            "--screenshot" => {
                screenshot_path = args.next();
            }
            "--exit" => {
                exit_after = true;
            }
            "--delay" => {
                if let Some(secs) = args.next() {
                    if let Ok(parsed) = secs.parse::<f64>() {
                        delay_secs = parsed;
                    }
                }
            }
            "--width" => {
                if let Some(w) = args.next() {
                    if let Ok(parsed) = w.parse::<u32>() {
                        width = parsed;
                    }
                }
            }
            "--height" => {
                if let Some(h) = args.next() {
                    if let Ok(parsed) = h.parse::<u32>() {
                        height = parsed;
                    }
                }
            }
            _ => {}
        }
    }

    let site_name = match site_name.as_deref() {
        Some(name) => {
            if sites::available_sites().contains(&name) {
                name.to_string()
            } else {
                eprintln!(
                    "Unknown site '{name}'. Valid sites: {}",
                    sites::available_sites().join(", ")
                );
                process::exit(1);
            }
        }
        None => {
            eprintln!("Usage: webpage-rust --site <NAME>");
            eprintln!("Available sites: {}", sites::available_sites().join(", "));
            process::exit(1);
        }
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut app::App::new(
            site_name,
            screenshot_path,
            exit_after,
            delay_secs,
            width,
            height,
        ))
        .unwrap();
}
