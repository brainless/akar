use std::path::{Path, PathBuf};
use std::process::Command;

use crate::capture_manifest::{CaptureEntry, MANIFEST};

pub struct CaptureResult {
    pub filename: String,
    pub success: bool,
    pub message: String,
}

pub struct CaptureConfig {
    pub output_dir: PathBuf,
    pub scripts_dir: PathBuf,
    pub images_dir: PathBuf,
    pub website_dir: PathBuf,
    pub delay: f32,
    pub dry_run: bool,
}

impl CaptureConfig {
    pub fn new(base_dir: &Path) -> Self {
        Self {
            output_dir: base_dir.join(".artifacts/epic026/captures"),
            scripts_dir: PathBuf::from("examples/demo-rust/scripts"),
            images_dir: PathBuf::from("images/components"),
            website_dir: PathBuf::from("website/public/screenshots/components"),
            delay: 0.5,
            dry_run: false,
        }
    }
}

fn parse_script_screenshot_path(script_path: &Path) -> Option<PathBuf> {
    let content = std::fs::read_to_string(script_path).ok()?;
    content
        .lines()
        .filter(|line| line.starts_with("screenshot "))
        .filter_map(|line| line.strip_prefix("screenshot ").map(PathBuf::from))
        .next_back()
}

fn build_command(entry: &CaptureEntry, config: &CaptureConfig) -> Vec<String> {
    let mut args = vec![
        "run".to_string(),
        "--bin".to_string(),
        "demo-rust".to_string(),
        "--".to_string(),
        "--component".to_string(),
        entry.component.to_string(),
    ];

    if let Some(variant) = entry.variant {
        args.push("--variant".to_string());
        args.push(variant.to_string());
    }

    if entry.state != "default" {
        args.push("--state".to_string());
        args.push(entry.state.to_string());
    }

    if let Some(script) = entry.script {
        let script_path = config.scripts_dir.join(script);
        args.push("--script".to_string());
        args.push(script_path.to_string_lossy().to_string());
    } else {
        let tmp_path = config.output_dir.join(entry.filename);
        args.push("--screenshot".to_string());
        args.push(tmp_path.to_string_lossy().to_string());
    }

    args.push("--delay".to_string());
    args.push(config.delay.to_string());

    args.push("--exit".to_string());

    args
}

fn is_flat_color(path: &Path) -> bool {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(_) => return false,
    };
    let decoder = png::Decoder::new(std::io::Cursor::new(data));
    let mut reader = match decoder.read_info() {
        Ok(r) => r,
        Err(_) => return false,
    };
    let info = reader.info().clone();
    let width = info.width as usize;
    let height = info.height as usize;
    if width == 0 || height == 0 {
        return true;
    }
    let mut buf = vec![0u8; reader.output_buffer_size()];
    if reader.next_frame(&mut buf).is_err() {
        return false;
    }
    let bytes_per_pixel = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Grayscale => 1,
        _ => return false,
    };
    let stride = width * bytes_per_pixel;
    let sample_count = 16.min(width).min(height);
    let mut samples = Vec::with_capacity(sample_count * sample_count);
    for sy in 0..sample_count {
        let row = (sy * height / sample_count) as usize;
        for sx in 0..sample_count {
            let col = (sx * width / sample_count) as usize;
            let idx = row * stride + col * bytes_per_pixel;
            if idx + bytes_per_pixel <= buf.len() {
                let r = buf[idx];
                let g = if bytes_per_pixel >= 3 {
                    buf[idx + 1]
                } else {
                    r
                };
                let b = if bytes_per_pixel >= 3 {
                    buf[idx + 2]
                } else {
                    r
                };
                samples.push((r as u32 + g as u32 + b as u32) / 3);
            }
        }
    }
    if samples.is_empty() {
        return true;
    }
    let mean: f32 = samples.iter().map(|&v| v as f32).sum::<f32>() / samples.len() as f32;
    let variance: f32 = samples
        .iter()
        .map(|&v| {
            let d = v as f32 - mean;
            d * d
        })
        .sum::<f32>()
        / samples.len() as f32;
    variance < 0.05
}

fn copy_with_verify(src: &Path, dst: &Path) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create dir {}: {e}", parent.display()))?;
    }
    let src_bytes = std::fs::read(src).map_err(|e| format!("read {}: {e}", src.display()))?;
    std::fs::write(dst, &src_bytes).map_err(|e| format!("write {}: {e}", dst.display()))?;
    let dst_bytes =
        std::fs::read(dst).map_err(|e| format!("read verify {}: {e}", dst.display()))?;
    if src_bytes != dst_bytes {
        return Err(format!(
            "byte mismatch: {} has {} bytes, {} has {} bytes",
            src.display(),
            src_bytes.len(),
            dst.display(),
            dst_bytes.len()
        ));
    }
    Ok(())
}

fn compare_against_baseline(
    current: &Path,
    baseline_dir: &Path,
    filename: &str,
) -> Result<(), String> {
    let baseline = baseline_dir.join(filename);
    if !baseline.exists() {
        return Err(format!("baseline not found: {}", baseline.display()));
    }
    let status = Command::new("akar-diff")
        .args([
            "--compare",
            baseline.to_str().unwrap(),
            current.to_str().unwrap(),
            "--threshold",
            "0.5",
        ])
        .status()
        .map_err(|e| format!("run akar-diff: {e}"))?;
    if !status.success() {
        return Err("akar-diff detected regression".to_string());
    }
    Ok(())
}

fn process_capture_result(
    entry: &CaptureEntry,
    config: &CaptureConfig,
    tmp_path: &std::path::Path,
    script_src: Option<&std::path::Path>,
) -> CaptureResult {
    if let Some(src) = script_src {
        if let Err(e) = copy_with_verify(src, tmp_path) {
            return CaptureResult {
                filename: entry.filename.to_string(),
                success: false,
                message: format!("copy script output from {}: {e}", src.display()),
            };
        }
    }

    if !tmp_path.exists() {
        return CaptureResult {
            filename: entry.filename.to_string(),
            success: false,
            message: "output file not created".to_string(),
        };
    }

    if is_flat_color(tmp_path) {
        let _ = std::fs::remove_file(tmp_path);
        return CaptureResult {
            filename: entry.filename.to_string(),
            success: false,
            message: "rejected: single flat color (variance near zero)".to_string(),
        };
    }

    let images_dst = config.images_dir.join(entry.filename);
    let website_dst = config.website_dir.join(entry.filename);

    if let Err(e) = copy_with_verify(tmp_path, &images_dst)
        .and_then(|()| copy_with_verify(tmp_path, &website_dst))
    {
        return CaptureResult {
            filename: entry.filename.to_string(),
            success: false,
            message: format!("copy failed: {e}"),
        };
    }

    if entry.is_regression {
        if let Err(e) = compare_against_baseline(tmp_path, &config.images_dir, entry.filename) {
            return CaptureResult {
                filename: entry.filename.to_string(),
                success: false,
                message: format!("regression: {e}"),
            };
        }
    }

    CaptureResult {
        filename: entry.filename.to_string(),
        success: true,
        message: "ok".to_string(),
    }
}

pub fn run_capture_all(config: &CaptureConfig) -> Vec<CaptureResult> {
    let _ = std::fs::create_dir_all(&config.output_dir);
    let _ = std::fs::create_dir_all(&config.images_dir);
    let _ = std::fs::create_dir_all(&config.website_dir);

    let mut results = Vec::with_capacity(MANIFEST.len());

    for entry in MANIFEST {
        if config.dry_run {
            results.push(CaptureResult {
                filename: entry.filename.to_string(),
                success: true,
                message: "dry run".to_string(),
            });
            continue;
        }

        let cmd_args = build_command(entry, config);
        let output = Command::new("cargo").args(&cmd_args).output();

        let tmp_path = config.output_dir.join(entry.filename);

        let result = match output {
            Ok(o) => {
                if !o.status.success() {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    CaptureResult {
                        filename: entry.filename.to_string(),
                        success: false,
                        message: format!(
                            "exit code {}: {}",
                            o.status.code().unwrap_or(-1),
                            stderr.trim()
                        ),
                    }
                } else {
                    let script_output = entry.script.and_then(|script| {
                        let script_path = config.scripts_dir.join(script);
                        parse_script_screenshot_path(&script_path)
                    });

                    let mut result =
                        process_capture_result(entry, config, &tmp_path, script_output.as_deref());

                    if !result.success && result.message.contains("flat color") {
                        let retry_output = Command::new("cargo").args(&cmd_args).output();

                        if let Ok(retry_o) = retry_output {
                            if retry_o.status.success() {
                                result = process_capture_result(
                                    entry,
                                    config,
                                    &tmp_path,
                                    script_output.as_deref(),
                                );
                                if result.success {
                                    result.message = "ok (retry)".to_string();
                                }
                            }
                        }
                    }

                    result
                }
            }
            Err(e) => CaptureResult {
                filename: entry.filename.to_string(),
                success: false,
                message: format!("spawn failed: {e}"),
            },
        };

        results.push(result);
    }

    results
}

pub fn verify_managed_dirs(config: &CaptureConfig) -> Result<(), String> {
    let mut expected: Vec<&str> = MANIFEST.iter().map(|e| e.filename).collect();
    expected.sort();

    for dir in [&config.images_dir, &config.website_dir] {
        let mut actual: Vec<String> = std::fs::read_dir(dir)
            .map_err(|e| format!("read {}: {e}", dir.display()))?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".png") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        actual.sort();

        let expected_strs: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        if actual != expected_strs {
            let missing: Vec<&str> = expected
                .iter()
                .filter(|f| !actual.contains(&f.to_string()))
                .copied()
                .collect();
            let unexpected: Vec<&String> = actual
                .iter()
                .filter(|f| !expected_strs.contains(f))
                .collect();
            let mut msg = String::new();
            if !missing.is_empty() {
                msg.push_str(&format!("missing: {:?}\n", missing));
            }
            if !unexpected.is_empty() {
                msg.push_str(&format!("unexpected: {:?}\n", unexpected));
            }
            return Err(format!("{} contents mismatch:\n{msg}", dir.display()));
        }
    }

    Ok(())
}

pub fn print_summary(results: &[CaptureResult]) {
    let ok = results.iter().filter(|r| r.success).count();
    let fail = results.iter().filter(|r| !r.success).count();
    println!("Capture complete: {ok} succeeded, {fail} failed");
    for r in results.iter().filter(|r| !r.success) {
        println!("  FAIL {}: {}", r.filename, r.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture_manifest::MANIFEST;

    #[test]
    fn build_command_basic() {
        let config = CaptureConfig {
            output_dir: PathBuf::from("/tmp/captures"),
            scripts_dir: PathBuf::from("scripts"),
            images_dir: PathBuf::from("images/components"),
            website_dir: PathBuf::from("website/public/screenshots/components"),
            delay: 0.5,
            dry_run: false,
        };
        let entry = &MANIFEST[0];
        let cmd = build_command(entry, &config);
        assert!(cmd.contains(&"--component".to_string()));
        assert!(cmd.contains(&"--exit".to_string()));
        assert!(cmd.contains(&"--screenshot".to_string()));
    }

    #[test]
    fn build_command_with_variant() {
        let config = CaptureConfig {
            output_dir: PathBuf::from("/tmp/captures"),
            scripts_dir: PathBuf::from("scripts"),
            images_dir: PathBuf::from("images/components"),
            website_dir: PathBuf::from("website/public/screenshots/components"),
            delay: 0.5,
            dry_run: false,
        };
        let entry = &MANIFEST[33];
        assert_eq!(entry.variant, Some("solid"));
        let cmd = build_command(entry, &config);
        assert!(cmd.contains(&"--variant".to_string()));
        assert!(cmd.contains(&"solid".to_string()));
    }

    #[test]
    fn build_command_with_script() {
        let config = CaptureConfig {
            output_dir: PathBuf::from("/tmp/captures"),
            scripts_dir: PathBuf::from("scripts"),
            images_dir: PathBuf::from("images/components"),
            website_dir: PathBuf::from("website/public/screenshots/components"),
            delay: 0.5,
            dry_run: false,
        };
        let entry = MANIFEST
            .iter()
            .find(|e| e.script.is_some())
            .expect("should have scripted entry");
        let cmd = build_command(entry, &config);
        assert!(cmd.contains(&"--script".to_string()));
        assert!(!cmd.contains(&"--screenshot".to_string()));
    }

    #[test]
    fn build_command_with_state() {
        let config = CaptureConfig {
            output_dir: PathBuf::from("/tmp/captures"),
            scripts_dir: PathBuf::from("scripts"),
            images_dir: PathBuf::from("images/components"),
            website_dir: PathBuf::from("website/public/screenshots/components"),
            delay: 0.5,
            dry_run: false,
        };
        let entry = MANIFEST
            .iter()
            .find(|e| e.state == "closable")
            .expect("should have closable entry");
        let cmd = build_command(entry, &config);
        assert!(cmd.contains(&"--state".to_string()));
        assert!(cmd.contains(&"closable".to_string()));
    }

    #[test]
    fn build_command_default_state_omitted() {
        let config = CaptureConfig {
            output_dir: PathBuf::from("/tmp/captures"),
            scripts_dir: PathBuf::from("scripts"),
            images_dir: PathBuf::from("images/components"),
            website_dir: PathBuf::from("website/public/screenshots/components"),
            delay: 0.5,
            dry_run: false,
        };
        let entry = &MANIFEST[0];
        assert_eq!(entry.state, "default");
        let cmd = build_command(entry, &config);
        assert!(!cmd.contains(&"--state".to_string()));
    }

    #[test]
    fn parse_script_screenshot_path_single() {
        let path = PathBuf::from("scripts/button_outline_hover.txt");
        let result = parse_script_screenshot_path(&path);
        assert_eq!(
            result,
            Some(PathBuf::from("/tmp/akar-button-outline-hover.png"))
        );
    }

    #[test]
    fn parse_script_screenshot_path_multiple() {
        let path = PathBuf::from("scripts/text_edit_paste.txt");
        let result = parse_script_screenshot_path(&path);
        assert_eq!(result, Some(PathBuf::from("/tmp/form-notes-pasted.png")));
    }
}
