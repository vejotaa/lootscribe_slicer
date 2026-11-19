use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SliceResult {
    pub printing_time_hours: f64,
    pub material_usage_grams: f64,
    pub gcode_path: String,
}

/// Executes Orca Slicer with proper profile loading and CLI flags,
/// then parses metrics from the generated .gcode file for reliable results.
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

    let _stdout = String::from_utf8_lossy(&output.stdout);
    // best-effort: parse orca output, but primary metrics come from .gcode file

    // Parse metrics from the generated .gcode file — more reliable than stdout scraping
    let gcode_path = input_path.with_extension("gcode");
    let (time_h, material_g) = parse_gcode_file(&gcode_path)?;

    let gcode_path_str = gcode_path.to_string_lossy().to_string();

    Ok(SliceResult {
        printing_time_hours: time_h,
        material_usage_grams: material_g,
        gcode_path: gcode_path_str,
    })
}

/// Parses G-code file footers/headers for print time and filament usage.
/// Looks for comments like:
///   ; estimated printing time (normal mode) = 1:23:45
///   ; total filament weight [g] = 45.3
fn parse_gcode_file(gcode_path: &Path) -> Result<(f64, f64)> {
    if !gcode_path.exists() {
        anyhow::bail!("Arquivo G-code não encontrado: {:?}. G-code pode não ter sido gerado ainda.", gcode_path);
    }

    let content = std::fs::read_to_string(gcode_path)
        .context("Falha ao ler arquivo G-code")?;

    let mut time_hours = 0.0;
    let mut material_grams = 0.0;

    for line in content.lines() {
        let lower = line.to_lowercase();
        if lower.contains("estimated printing time") || lower.contains("print time") {
            if let Some(val) = extract_gcode_time(line) {
                time_hours = val;
            }
        }
        if lower.contains("total filament weight") || lower.contains("filament used") {
            if let Some(val) = extract_gcode_filament(line) {
                material_grams = val;
            }
        }
    }

    if time_hours == 0.0 && material_grams == 0.0 {
        anyhow::bail!(
            "Não foi possível extrair tempo/material do G-code em {:?}",
            gcode_path
        );
    }

    Ok((time_hours, material_grams))
}

/// Extracts print time from a G-code comment line.
/// Expected format: `; estimated printing time (normal mode) = 1:23:45`
/// or `; print time: 1h 23m 45s`
fn extract_gcode_time(line: &str) -> Option<f64> {
    let lower = line.to_lowercase();

    // Format: ; estimated printing time (normal mode) = 1:23:45
    if let Some(rest) = lower.split("= ").nth(1) {
        let trimmed = rest.trim();
        if trimmed.contains(':') {
            let time_parts: Vec<&str> = trimmed.split(':').collect();
            if time_parts.len() == 3 {
                if let (Ok(h), Ok(m), Ok(s)) = (
                    time_parts[0].parse::<f64>(),
                    time_parts[1].parse::<f64>(),
                    time_parts[2].parse::<f64>(),
                ) {
                    let total_seconds = h * 3600.0 + m * 60.0 + s;
                    return Some(total_seconds / 3600.0); // return in hours
                }
            }
        } else {
            let mut hours = 0.0f64;
            let mut minutes = 0.0f64;
            let mut seconds = 0.0f64;
            let mut found_h = false;
            let mut found_m = false;
            let mut found_s = false;

            for part in trimmed.split_whitespace() {
                if part.ends_with('h') && !found_h {
                    let num: f64 = part.trim_end_matches('h').parse().unwrap_or(0.0);
                    hours = num;
                    found_h = true;
                } else if part.ends_with('m') && !found_m {
                    let num: f64 = part.trim_end_matches('m').parse().unwrap_or(0.0);
                    minutes = num;
                    found_m = true;
                } else if part.ends_with('s') && !found_s {
                    let num: f64 = part.trim_end_matches('s').parse().unwrap_or(0.0);
                    seconds = num;
                    found_s = true;
                }
            }

            if found_h || found_m || found_s {
                return Some(hours + minutes / 60.0 + seconds / 3600.0);
            }
        }
    }

    // Format: ; print time: 1h 23m 45s
    if lower.contains("print time:") {
        let mut hours = 0.0f64;
        let mut minutes = 0.0f64;
        let mut seconds = 0.0f64;
        let mut found_h = false;
        let mut found_m = false;
        let mut found_s = false;

        for part in lower.split_whitespace() {
            if part.ends_with('h') && !found_h {
                let num: f64 = part.trim_end_matches('h').parse().unwrap_or(0.0);
                hours = num;
                found_h = true;
            } else if part.ends_with('m') && !found_m {
                let num: f64 = part.trim_end_matches('m').parse().unwrap_or(0.0);
                minutes = num;
                found_m = true;
            } else if part.ends_with('s') && !found_s {
                let num: f64 = part.trim_end_matches('s').parse().unwrap_or(0.0);
                seconds = num;
                found_s = true;
            }
        }

        if found_h || found_m || found_s {
            return Some(hours + minutes / 60.0 + seconds / 3600.0);
        }
    }

    None
}

/// Extracts filament weight from a G-code comment line.
/// Expected format: `; total filament weight [g] = 45.3` or `; total filament weight [kg] = 0.045`
fn extract_gcode_filament(line: &str) -> Option<f64> {
    let lower = line.to_lowercase();

    // Format: ; total filament weight [g] = 45.3
    if let Some(rest) = lower.split("= ").nth(1) {
        let val = rest.trim();
        let num: f64 = val.parse().ok()?;
        if num > 0.0 {
            if lower.contains("[kg]") {
                return Some(num * 1000.0); // convert kg to g
            }
            return Some(num);
        }
    }

    // Format: ; filament used: 45.3g or ; filament used: 0.045kg
    if lower.contains("filament used:") || lower.contains("filament:") {
        if let Some(rest) = lower.split(":").nth(1) {
            let rest_owned = rest.to_string();
            if rest_owned.to_lowercase().contains("kg") {
                let num_str = rest_owned.replace("kg", "");
                let num: f64 = num_str.trim().parse().ok()?;
                if num > 0.0 {
                    return Some(num * 1000.0); // convert kg to grams
                }
            } else {
                let num_str = rest_owned.replace("g", "");
                let num: f64 = num_str.trim().parse().ok()?;
                if num > 0.0 {
                    return Some(num); // already in grams
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_extract_gcode_time_dms_format() {
        // Format: ; estimated printing time (normal mode) = 1:23:45
        let line = "; estimated printing time (normal mode) = 1:23:45";
        let time = extract_gcode_time(line);
        assert!(time.is_some(), "Expected time to be parsed, got none");
        let t = time.unwrap();
        // 1*3600 + 23*60 + 45 = 5025 seconds = 1.3958 hours
        assert!((t - 1.3958).abs() < 0.01);
    }

    #[test]
    fn test_extract_gcode_time_hms_format() {
        // Format: ; print time: 1h 23m 45s
        let line = "; print time: 1h 23m 45s";
        let time = extract_gcode_time(line);
        assert!(time.is_some());
        let t = time.unwrap();
        assert!((t - 1.3958).abs() < 0.01);
    }

    #[test]
    fn test_extract_gcode_time_h_only() {
        let line = "; print time: 2h";
        let time = extract_gcode_time(line);
        assert!(time.is_some());
        assert!((time.unwrap() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_extract_gcode_time_m_only() {
        let line = "; print time: 30m";
        let time = extract_gcode_time(line);
        assert!(time.is_some());
        assert!((time.unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_extract_gcode_filament_g_format() {
        // Format: ; total filament weight [g] = 45.3
        let line = "; total filament weight [g] = 45.3";
        let weight = extract_gcode_filament(line);
        assert!(weight.is_some());
        let w = weight.unwrap();
        assert!((w - 45.3).abs() < 0.1);
    }

    #[test]
    fn test_extract_gcode_filament_kg_format() {
        // Format: ; total filament weight [kg] = 0.045
        let line = "; total filament weight [kg] = 0.045";
        let weight = extract_gcode_filament(line);
        assert!(weight.is_some());
        let w = weight.unwrap();
        assert!((w - 45.0).abs() < 0.1); // should convert to grams
    }

    #[test]
    fn test_extract_gcode_filament_no_value() {
        let line = "; some comment about filament";
        let weight = extract_gcode_filament(line);
        assert!(weight.is_none() || weight.unwrap_or(0.0) == 0.0);
    }

    #[test]
    fn test_parse_gcode_file_with_valid_content() {
        // Create a temporary gcode file for testing
        let tmp_dir = std::env::temp_dir();
        let gcode_path = tmp_dir.join("test_slice_ut.gcode");
        let content = r#"; G-code generated by Orca Slicer
; estimated printing time (normal mode) = 1:30:00
; total filament weight [g] = 35.5
; some other comment
"#;
        fs::write(&gcode_path, content).expect("Failed to write test gcode");
        let result = parse_gcode_file(&gcode_path);
        // Clean up
        let _ = fs::remove_file(&gcode_path);

        assert!(result.is_ok(), "parse_gcode_file failed: {:?}", result.err());
        let (time, material) = result.unwrap();
        assert!(time > 0.0, "Time should be > 0");
        assert!(material > 0.0, "Material should be > 0");
        // 1:30:00 = 1h 30m = 90 min = 1.5 hours
        assert!((time - 1.5).abs() < 0.1, "Expected ~1.5h, got {}", time);
        assert!((material - 35.5).abs() < 0.1, "Expected ~35.5g, got {}", material);
    }

    #[test]
    fn test_slice_result_creation() {
        let result = SliceResult {
            printing_time_hours: 1.5,
            material_usage_grams: 35.5,
            gcode_path: "/path/to/file.gcode".to_string(),
        };
        assert!((result.printing_time_hours - 1.5).abs() < f64::EPSILON);
        assert!((result.material_usage_grams - 35.5).abs() < f64::EPSILON);
        assert_eq!(result.gcode_path, "/path/to/file.gcode");
    }

    #[test]
    fn test_parse_gcode_file_missing_file() {
        let non_existent = Path::new("/tmp/nonexistent_gcode_file_gcode123.gcode");
        let result = parse_gcode_file(non_existent);
        assert!(result.is_err());
    }
}