use analyzer::csv_reader::KeyFreq;
use analyzer::geometry::{Geometry, GeometryName, visualization::save_layout_with_layers};
use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    env_logger::init();

    let geometries = [
        (GeometryName::RowStagger, "row_stagger"),
        (GeometryName::Ortho, "ortho"),
    ];

    for (geometry_name, file_prefix) in &geometries {
        let geom = Geometry::build(*geometry_name)?;

        // 空の頻度データ（表示テスト用）
        let key_freq = KeyFreq::new();

        // レイヤ記号のサンプルデータを作成
        // (symbol, layer_number, modifier_key)
        let layer_symbols = vec![
            // Base layer (layer 0) - 通常のアルファベット
            ("a".to_string(), 0, "".to_string()),
            ("s".to_string(), 0, "".to_string()),
            ("d".to_string(), 0, "".to_string()),
            ("f".to_string(), 0, "".to_string()),
            ("j".to_string(), 0, "".to_string()),
            ("k".to_string(), 0, "".to_string()),
            ("l".to_string(), 0, "".to_string()),
            // Layer 1 - 記号キー (LThumb modifier)
            ("!".to_string(), 1, "LThumb".to_string()),
            ("@".to_string(), 1, "LThumb".to_string()),
            ("#".to_string(), 1, "LThumb".to_string()),
            ("$".to_string(), 1, "LThumb".to_string()),
            ("&".to_string(), 1, "LThumb".to_string()),
            ("*".to_string(), 1, "LThumb".to_string()),
            ("(".to_string(), 1, "LThumb".to_string()),
            // Layer 2 - 数字キー (RThumb modifier)
            ("1".to_string(), 2, "RThumb".to_string()),
            ("2".to_string(), 2, "RThumb".to_string()),
            ("3".to_string(), 2, "RThumb".to_string()),
            ("4".to_string(), 2, "RThumb".to_string()),
            ("7".to_string(), 2, "RThumb".to_string()),
            ("8".to_string(), 2, "RThumb".to_string()),
            ("9".to_string(), 2, "RThumb".to_string()),
        ];

        let output_path =
            save_layout_with_layers(&geom, &key_freq, true, file_prefix, &layer_symbols)?;
        info!("Wrote layer visualization: {}", output_path.display());
    }

    info!("All layer visualizations generated in figs/ directory");
    Ok(())
}
