use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::capture_manifest::{CaptureEntry, MANIFEST};

pub struct CaptureResult {
    pub filename: String,
    pub success: bool,
    pub message: String,
}

pub struct CaptureConfig {
    pub workspace_dir: PathBuf,
    pub output_dir: PathBuf,
    pub baseline_dir: PathBuf,
    pub diff_dir: PathBuf,
    pub scripts_dir: PathBuf,
    pub images_dir: PathBuf,
    pub website_dir: PathBuf,
    pub delay: f32,
    pub dry_run: bool,
}

impl CaptureConfig {
    pub fn new(base_dir: &Path) -> Self {
        let workspace_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("demo-rust must be inside the akar workspace")
            .to_path_buf();
        let artifact_dir = base_dir.join(".artifacts/epic026");
        Self {
            output_dir: artifact_dir.join("captures"),
            baseline_dir: artifact_dir.join("baselines"),
            diff_dir: artifact_dir.join("diffs"),
            scripts_dir: workspace_dir.join("examples/demo-rust/scripts"),
            images_dir: workspace_dir.join("images/components"),
            website_dir: workspace_dir.join("website/public/screenshots/components"),
            workspace_dir,
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
    // Sample every pixel rather than a coarse fixed-size lattice: a sparse
    // lattice (e.g. 16x16) can systematically miss small, legitimate content
    // (a short caption, a thin icon) and misreport a real frame as a flat
    // cold-start capture. Scanning the full buffer costs little at these
    // resolutions and only flags frames that truly carry no variance.
    let mut samples = Vec::with_capacity(width * height);
    for row in 0..height {
        for col in 0..width {
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

fn akar_diff_args(config: &CaptureConfig, args: &[&Path], mode: &str) -> Vec<OsString> {
    let mut command = vec![
        OsString::from("run"),
        OsString::from("--quiet"),
        OsString::from("--manifest-path"),
        config.workspace_dir.join("Cargo.toml").into_os_string(),
        OsString::from("--bin"),
        OsString::from("akar-diff"),
        OsString::from("--"),
        OsString::from(mode),
    ];
    command.extend(args.iter().map(|path| path.as_os_str().to_owned()));
    command
}

fn run_akar_diff(args: &[OsString]) -> Result<std::process::Output, String> {
    Command::new("cargo")
        .args(args)
        .output()
        .map_err(|e| format!("run workspace akar-diff through cargo: {e}"))
}

fn command_output(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    [stdout.trim(), stderr.trim()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

fn compare_against_baseline(
    current: &Path,
    config: &CaptureConfig,
    filename: &str,
) -> Result<(), String> {
    let baseline = config.baseline_dir.join(filename);
    if !baseline.exists() {
        return Err(format!(
            "baseline not found: {} (restore the Task 0 pre-change baseline before capture)",
            baseline.display()
        ));
    }

    let mut compare_args = akar_diff_args(config, &[&baseline, current], "--compare");
    compare_args.extend([OsString::from("--threshold"), OsString::from("0")]);
    let compare_output = run_akar_diff(&compare_args)?;
    if compare_output.status.success() {
        return Ok(());
    }

    std::fs::create_dir_all(&config.diff_dir)
        .map_err(|e| format!("create diff directory {}: {e}", config.diff_dir.display()))?;
    let diff_path = config.diff_dir.join(filename);
    let mut diff_args = akar_diff_args(config, &[&baseline, current], "--diff");
    diff_args.extend([OsString::from("-o"), diff_path.as_os_str().to_owned()]);
    let diff_output = run_akar_diff(&diff_args)?;

    let comparison = command_output(&compare_output);
    if diff_output.status.success() {
        Err(format!(
            "pixel regression against {}; visual diff: {}; {}",
            baseline.display(),
            diff_path.display(),
            comparison
        ))
    } else {
        Err(format!(
            "pixel regression against {}; comparison: {}; visual diff failed: {}",
            baseline.display(),
            comparison,
            command_output(&diff_output)
        ))
    }
}

fn process_capture_result_with_compare(
    entry: &CaptureEntry,
    config: &CaptureConfig,
    tmp_path: &std::path::Path,
    script_src: Option<&std::path::Path>,
    compare: &dyn Fn(&Path, &CaptureConfig, &str) -> Result<(), String>,
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

    if entry.is_regression {
        if let Err(e) = compare(tmp_path, config, entry.filename) {
            return CaptureResult {
                filename: entry.filename.to_string(),
                success: false,
                message: format!("regression: {e}"),
            };
        }
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

    CaptureResult {
        filename: entry.filename.to_string(),
        success: true,
        message: "ok".to_string(),
    }
}

fn process_capture_result(
    entry: &CaptureEntry,
    config: &CaptureConfig,
    tmp_path: &Path,
    script_src: Option<&Path>,
) -> CaptureResult {
    process_capture_result_with_compare(
        entry,
        config,
        tmp_path,
        script_src,
        &compare_against_baseline,
    )
}

fn missing_regression_baselines(config: &CaptureConfig) -> Vec<CaptureResult> {
    MANIFEST
        .iter()
        .filter(|entry| entry.is_regression)
        .filter_map(|entry| {
            let baseline = config.baseline_dir.join(entry.filename);
            (!baseline.is_file()).then(|| CaptureResult {
                filename: entry.filename.to_string(),
                success: false,
                message: format!(
                    "baseline not found: {} (restore the Task 0 pre-change baseline set before capture)",
                    baseline.display()
                ),
            })
        })
        .collect()
}

pub fn run_capture_all(config: &CaptureConfig) -> Vec<CaptureResult> {
    if !config.dry_run {
        let missing_baselines = missing_regression_baselines(config);
        if !missing_baselines.is_empty() {
            return missing_baselines;
        }
    }

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
    use crate::cli;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn test_config() -> CaptureConfig {
        CaptureConfig {
            workspace_dir: PathBuf::from("/workspace"),
            output_dir: PathBuf::from("/tmp/captures"),
            baseline_dir: PathBuf::from("/tmp/baselines"),
            diff_dir: PathBuf::from("/tmp/diffs"),
            scripts_dir: PathBuf::from("scripts"),
            images_dir: PathBuf::from("images/components"),
            website_dir: PathBuf::from("website/public/screenshots/components"),
            delay: 0.5,
            dry_run: false,
        }
    }

    fn unique_test_dir() -> PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "akar-capture-runner-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn write_non_flat_png(path: &Path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let file = std::fs::File::create(path).unwrap();
        let mut encoder = png::Encoder::new(file, 2, 2);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer
            .write_image_data(&[
                0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
            ])
            .unwrap();
    }

    #[test]
    fn build_command_basic() {
        let config = test_config();
        let entry = &MANIFEST[0];
        let cmd = build_command(entry, &config);
        assert!(cmd.contains(&"--component".to_string()));
        assert!(cmd.contains(&"--exit".to_string()));
        assert!(cmd.contains(&"--screenshot".to_string()));
    }

    #[test]
    fn build_command_with_variant() {
        let config = test_config();
        let entry = &MANIFEST[33];
        assert_eq!(entry.variant, Some("solid"));
        let cmd = build_command(entry, &config);
        assert!(cmd.contains(&"--variant".to_string()));
        assert!(cmd.contains(&"solid".to_string()));
    }

    #[test]
    fn build_command_with_script() {
        let config = test_config();
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
        let config = test_config();
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
        let config = test_config();
        let entry = &MANIFEST[0];
        assert_eq!(entry.state, "default");
        let cmd = build_command(entry, &config);
        assert!(!cmd.contains(&"--state".to_string()));
    }

    #[test]
    fn every_manifest_command_passes_cli_validation() {
        let config = test_config();

        for entry in MANIFEST {
            let command = build_command(entry, &config);
            let separator = command
                .iter()
                .position(|arg| arg == "--")
                .expect("cargo command should contain argument separator");
            let args = std::iter::once("demo-rust".to_string())
                .chain(command[separator + 1..].iter().cloned());
            if let Err(error) = cli::parse(args) {
                panic!(
                    "manifest command for '{}' failed CLI validation: {error}",
                    entry.filename
                );
            }
        }
    }

    #[test]
    fn config_uses_durable_epic_baseline_and_diff_directories() {
        let base = PathBuf::from("capture-root");
        let config = CaptureConfig::new(&base);
        assert_eq!(
            config.baseline_dir,
            base.join(".artifacts/epic026/baselines")
        );
        assert_eq!(config.diff_dir, base.join(".artifacts/epic026/diffs"));
        assert!(config.images_dir.is_absolute());
        assert!(config.website_dir.is_absolute());
    }

    #[test]
    fn akar_diff_is_invoked_from_the_workspace_with_pixel_exact_threshold() {
        let config = test_config();
        let baseline = Path::new("baseline.png");
        let current = Path::new("current.png");
        let mut args = akar_diff_args(&config, &[baseline, current], "--compare");
        args.extend([OsString::from("--threshold"), OsString::from("0")]);
        assert_eq!(args[0], "run");
        assert_eq!(args[1], "--quiet");
        assert_eq!(args[2], "--manifest-path");
        assert_eq!(args[3], config.workspace_dir.join("Cargo.toml"));
        assert_eq!(args[4], "--bin");
        assert_eq!(args[5], "akar-diff");
        assert_eq!(args[6], "--");
        assert_eq!(args[7], "--compare");
        assert_eq!(args[8], baseline);
        assert_eq!(args[9], current);
        assert_eq!(args[10], "--threshold");
        assert_eq!(args[11], "0");
    }

    #[test]
    fn failed_regression_comparison_does_not_overwrite_managed_outputs() {
        let root = unique_test_dir();
        let mut config = test_config();
        config.output_dir = root.join("captures");
        config.baseline_dir = root.join("baselines");
        config.diff_dir = root.join("diffs");
        config.images_dir = root.join("images");
        config.website_dir = root.join("website");

        let entry = MANIFEST
            .iter()
            .find(|entry| entry.is_regression)
            .expect("manifest should include a regression capture");
        let current = config.output_dir.join(entry.filename);
        write_non_flat_png(&current);

        let images_dst = config.images_dir.join(entry.filename);
        let website_dst = config.website_dir.join(entry.filename);
        std::fs::create_dir_all(&config.images_dir).unwrap();
        std::fs::create_dir_all(&config.website_dir).unwrap();
        std::fs::write(&images_dst, b"old images output").unwrap();
        std::fs::write(&website_dst, b"old website output").unwrap();

        let result =
            process_capture_result_with_compare(entry, &config, &current, None, &|_, _, _| {
                Err("changed pixels".to_string())
            });

        assert!(!result.success);
        assert!(result.message.contains("changed pixels"));
        assert_eq!(std::fs::read(&images_dst).unwrap(), b"old images output");
        assert_eq!(std::fs::read(&website_dst).unwrap(), b"old website output");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_baselines_fail_preflight_before_capture() {
        let root = unique_test_dir();
        let mut config = test_config();
        config.baseline_dir = root.join("baselines");
        let failures = missing_regression_baselines(&config);
        assert_eq!(failures.len(), crate::capture_manifest::regression_count());
        assert!(failures.iter().all(|failure| !failure.success));
        assert!(failures
            .iter()
            .all(|failure| failure.message.contains("baseline not found")));
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
