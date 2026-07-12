use anyhow::{anyhow, Result};
use std::process::ExitCode;

struct Image {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn decode_png(path: &str) -> Result<Image> {
    let file = std::fs::File::open(path).map_err(|e| anyhow!("failed to open '{path}': {e}"))?;
    let mut reader = png::Decoder::new(file).read_info()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());
    if info.color_type != png::ColorType::Rgba {
        return Err(anyhow!("'{path}' is {:?}, expected RGBA", info.color_type));
    }
    Ok(Image {
        width: info.width,
        height: info.height,
        rgba: buf,
    })
}

fn encode_png(path: &str, img: &Image) -> Result<()> {
    let file =
        std::fs::File::create(path).map_err(|e| anyhow!("failed to create '{path}': {e}"))?;
    let mut encoder = png::Encoder::new(file, img.width, img.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&img.rgba)?;
    Ok(())
}

fn build_diff(base: &Image, cur: &Image) -> Vec<u8> {
    let mut out = Vec::with_capacity(base.rgba.len());
    for px in base.rgba.chunks_exact(4).zip(cur.rgba.chunks_exact(4)) {
        if px.0 != px.1 {
            out.extend_from_slice(&[255, 0, 0, 255]);
        } else {
            let dim = |c: u8| (c as f32 * 0.3) as u8;
            out.extend_from_slice(&[dim(px.0[0]), dim(px.0[1]), dim(px.0[2]), 255]);
        }
    }
    out
}

fn changed_count(base: &Image, cur: &Image) -> usize {
    base.rgba
        .chunks_exact(4)
        .zip(cur.rgba.chunks_exact(4))
        .filter(|px| px.0 != px.1)
        .count()
}

fn same_size(base: &Image, cur: &Image) -> Result<()> {
    if base.width != cur.width || base.height != cur.height {
        return Err(anyhow!(
            "size mismatch: baseline {}x{} vs current {}x{}",
            base.width,
            base.height,
            cur.width,
            cur.height
        ));
    }
    Ok(())
}

fn parse_two(paths: &[String]) -> Result<(String, String)> {
    let mut it = paths.iter();
    let base = it
        .next()
        .cloned()
        .ok_or_else(|| anyhow!("missing baseline path"))?;
    let cur = it
        .next()
        .cloned()
        .ok_or_else(|| anyhow!("missing current path"))?;
    Ok((base, cur))
}

fn run_diff(args: &[String]) -> Result<()> {
    let mut base_cur: Vec<String> = Vec::new();
    let mut out_path = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                out_path = args.get(i + 1).cloned();
                i += 2;
            }
            other => {
                base_cur.push(other.to_string());
                i += 1;
            }
        }
    }
    let (base_path, cur_path) = parse_two(&base_cur)?;
    let out_path = out_path.ok_or_else(|| anyhow!("missing output path (-o)"))?;
    let base = decode_png(&base_path)?;
    let cur = decode_png(&cur_path)?;
    same_size(&base, &cur)?;
    let rgba = build_diff(&base, &cur);
    encode_png(
        &out_path,
        &Image {
            width: base.width,
            height: base.height,
            rgba,
        },
    )?;
    println!("diff written to {out_path}");
    Ok(())
}

fn run_compare(args: &[String]) -> Result<ExitCode> {
    let mut base_cur: Vec<String> = Vec::new();
    let mut threshold = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--threshold" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| anyhow!("--threshold missing value"))?;
                threshold = Some(
                    v.parse::<f64>()
                        .map_err(|e| anyhow!("invalid threshold: {e}"))?,
                );
                i += 2;
            }
            other => {
                base_cur.push(other.to_string());
                i += 1;
            }
        }
    }
    let (base_path, cur_path) = parse_two(&base_cur)?;
    let threshold = threshold.ok_or_else(|| anyhow!("missing --threshold PCT"))?;
    let base = decode_png(&base_path)?;
    let cur = decode_png(&cur_path)?;
    same_size(&base, &cur)?;
    let total = base.width as usize * base.height as usize;
    let changed = changed_count(&base, &cur);
    let pct = changed as f64 / total as f64 * 100.0;
    if pct > threshold {
        println!("FAIL: {changed}/{total} changed ({pct:.2}% > {threshold}% threshold)");
        return Ok(ExitCode::FAILURE);
    }
    println!("PASS: {changed}/{total} changed ({pct:.2}% <= {threshold}% threshold)");
    Ok(ExitCode::SUCCESS)
}

fn usage() {
    eprintln!("akar-diff - diff/compare PNGs from the screenshot tool");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  akar-diff --diff BASE CUR -o OUT.png");
    eprintln!("  akar-diff --compare BASE CUR --threshold PCT");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
        return ExitCode::FAILURE;
    }
    let result = match args[0].as_str() {
        "--diff" => run_diff(&args[1..]).map(|_| ExitCode::SUCCESS),
        "--compare" => run_compare(&args[1..]),
        _ => {
            usage();
            Ok(ExitCode::FAILURE)
        }
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("akar-diff: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(width: u32, height: u32, fill: u8) -> Image {
        Image {
            width,
            height,
            rgba: vec![fill; (width * height * 4) as usize],
        }
    }

    #[test]
    fn diff_marks_changed_red_and_dims_unchanged() {
        let base = img(2, 1, 100);
        let mut cur = img(2, 1, 100);
        cur.rgba[4..8].copy_from_slice(&[200, 200, 200, 255]);
        let out = build_diff(&base, &cur);
        assert_eq!(&out[0..4], &[30, 30, 30, 255]);
        assert_eq!(&out[4..8], &[255, 0, 0, 255]);
    }

    #[test]
    fn changed_count_is_pixel_exact() {
        let base = img(2, 1, 0);
        let mut cur = img(2, 1, 0);
        cur.rgba[0] = 1;
        cur.rgba[7] = 254;
        assert_eq!(changed_count(&base, &cur), 2);
    }

    #[test]
    fn compare_roundtrip_via_png() {
        let dir = std::env::temp_dir();
        let base = dir.join("akar-diff-test-base.png");
        let cur = dir.join("akar-diff-test-cur.png");
        let b = img(2, 2, 50);
        let mut c = img(2, 2, 50);
        c.rgba[8..12].copy_from_slice(&[9, 9, 9, 255]);
        encode_png(base.to_str().unwrap(), &b).unwrap();
        encode_png(cur.to_str().unwrap(), &c).unwrap();
        let db = decode_png(base.to_str().unwrap()).unwrap();
        let dc = decode_png(cur.to_str().unwrap()).unwrap();
        assert_eq!(changed_count(&db, &dc), 1);
        let _ = std::fs::remove_file(&base);
        let _ = std::fs::remove_file(&cur);
    }
}
