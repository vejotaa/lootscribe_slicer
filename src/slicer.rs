use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SliceResult {
    pub printing_time_hours: f64,
    pub material_usage_grams: f64,
    pub gcode_path: String,
}

pub fn slice_with_orca(
    input_file: &str,
    config_dir: &str,
    no_orient: bool,
    no_arrange: bool,
) -> Result<SliceResult> {
    let mut args = vec![
        "--load".to_string(),
        config_dir.to_string(),
        "--input".to_string(),
        input_file.to_string(),
    ];

    if !no_orient {
        args.push("--orient".to_string());
        args.push("1".to_string());
    }
    if !no_arrange {
        args.push("--arrange".to_string());
        args.push("1".to_string());
    }

    let output = Command::new("orca-slicer")
        .args(&args)
        .output()
        .context("Falha ao executar orca-slicer")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("orca-slicer falhou: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let (time_h, material_g) = parse_orca_output(&stdout)?;

    let input_path = Path::new(input_file);
    let gcode_name = input_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string() + ".gcode";
    let gcode_path = input_path.parent().unwrap_or(Path::new(".")).join(&gcode_name);
    let gcode_path_str = gcode_path.to_string_lossy().to_string();

    Ok(SliceResult {
        printing_time_hours: time_h,
        material_usage_grams: material_g,
        gcode_path: gcode_path_str,
    })
}

fn parse_orca_output(output: &str) -> Result<(f64, f64)> {
    let mut time_hours = 0.0;
    let mut material_grams = 0.0;

    for line in output.lines() {
        if line.contains("Print time:") {
            if let Some(val) = line.split(':').nth(1) {
                time_hours = parse_time_to_hours(val.trim())?;
            }
        } else if line.contains("Filament used:") {
            if let Some(val) = line.split(':').nth(1) {
                material_grams = parse_filament_to_grams(val.trim())?;
            }
        }
    }

    if time_hours == 0.0 || material_grams == 0.0 {
        anyhow::bail!("Não foi possível extrair tempo/material da saída do OrcaSlicer");
    }

    Ok((time_hours, material_grams))
}

fn parse_time_to_hours(s: &str) -> Result<f64> {
    let s = s.to_lowercase();
    let mut total_minutes = 0.0;

    for part in s.split_whitespace() {
        if part.ends_with('h') {
            total_minutes += part.trim_end_matches('h').parse::<f64>()? * 60.0;
        } else if part.ends_with('m') {
            total_minutes += part.trim_end_matches('m').parse::<f64>()?;
        } else if part.ends_with('s') {
            total_minutes += part.trim_end_matches('s').parse::<f64>()? / 60.0;
        }
    }

    Ok(total_minutes / 60.0)
}

fn parse_filament_to_grams(s: &str) -> Result<f64> {
    let s = s.replace(',', "");
    if s.ends_with("kg") {
        Ok(s.trim_end_matches("kg").trim().parse::<f64>()? * 1000.0)
    } else if s.ends_with('g') {
        Ok(s.trim_end_matches('g').trim().parse::<f64>()?)
    } else {
        Ok(s.trim().parse::<f64>()?)
    }
}