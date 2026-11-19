use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SliceResult {
    pub printing_time_hours: f64,
    pub material_usage_grams: f64,
    pub gcode_path: String,
}

/// Executes Orca Slicer with proper profile loading and CLI flags
pub fn slice_with_orca(
    input_file: &str,
    config_dir: &str,
    no_orient: bool,
    no_arrange: bool,
) -> Result<SliceResult> {
    let input_path = Path::new(input_file);

    // Build argument list following Orca Slicer CLI conventions
    let mut args = vec![
        "--config-dir".to_string(),
        config_dir.to_string(),
        "--outputdir".to_string(),
        input_path.parent()
            .unwrap_or(Path::new("."))
            .to_string_lossy()
            .to_string(),
        "--load-settings".to_string(),
        format!("{}/machine.k1c.opc", config_dir),
        "--load-filaments".to_string(),
        format!("{}/filament.k1c.opc", config_dir),
        "--slice".to_string(),
        "0".to_string(),
        "--input".to_string(),
        input_file.to_string(),
    ];

    // Conditional flags based on user preferences
    if !no_orient {
        args.push("--orient".to_string());
        args.push("1".to_string());
    }
    if !no_arrange {
        args.push("--arrange".to_string());
        args.push("1".to_string());
    }

    // Execute Orca Slicer
    let output = Command::new("orca-slicer")
        .args(&args)
        .output()
        .context("Falha ao executar orca-slicer. Verifique se o binary está no PATH e o config-dir está correto.")?;

    if output.status.code() != Some(0) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("orca-slicer retornou código de saída não-zero: {}\nStderr: {}", output.status, stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let (time_h, material_g) = parse_orca_output(&stdout)?;

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
        if line.contains("Print time:") || line.contains("Estimated printing time:") {
            if let Some(val) = line.split(':').nth(1) {
                time_hours = parse_time_to_hours(val.trim())?;
            }
        } else if line.contains("Filament used:") || line.contains("total filament weight") {
            if let Some(val) = line.split(':').nth(1) {
                material_grams = parse_filament_to_grams(val.trim())?;
            }
        }
    }

    if time_hours == 0.0 && material_grams == 0.0 {
        anyhow::bail!("Não foi possível extrair tempo/material da saída do OrcaSlicer. Verifique o arquivo de entrada e o perfil configurado.");
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
    if s.to_lowercase().ends_with("kg") {
        Ok(s.trim_end_matches("kg").trim().parse::<f64>()? * 1000.0)
    } else if s.to_lowercase().ends_with('g') {
        Ok(s.trim_end_matches('g').trim().parse::<f64>()?)
    } else {
        Ok(s.trim().parse::<f64>()?)
    }
}