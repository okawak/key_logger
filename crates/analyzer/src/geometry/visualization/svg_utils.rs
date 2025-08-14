use std::fs;
use std::path::{Path, PathBuf};

use super::super::types::Geometry;
use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;

/// HTMLエンコード（SVGテキスト用）
pub fn html_encode(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}

/// figsディレクトリに最適化レイアウトを保存
pub fn save_optimized_layout_to_figs(
    geom: &Geometry,
    freqs: &KeyFreq,
) -> Result<PathBuf, KbOptError> {
    let output_dir = "figs";
    fs::create_dir_all(output_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let filename = format!("optimized_{:?}_{}.svg", geom.name, timestamp)
        .to_lowercase()
        .replace(" ", "_");

    let output_path = Path::new(output_dir).join(&filename);

    super::layout_renderer::render_optimized_layout(geom, freqs, &output_path)?;

    println!("Optimized layout saved to: {}", output_path.display());
    Ok(output_path)
}

/// 指定パスに最適化レイアウトを保存
pub fn save_optimized_layout<P: AsRef<Path>>(
    geom: &Geometry,
    freqs: &KeyFreq,
    output_path: P,
) -> Result<(), KbOptError> {
    super::layout_renderer::render_optimized_layout(geom, freqs, output_path)
}
