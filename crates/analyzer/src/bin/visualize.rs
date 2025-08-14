use analyzer::csv_reader::KeyFreq;
use analyzer::geometry::{Geometry, GeometryName, render_optimized_layout};
use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    // figsディレクトリが存在しない場合は作成
    fs::create_dir_all("figs")?;
    let geometries = [
        (GeometryName::RowStagger, "row_stagger"),
        (GeometryName::Ortho, "ortho"),
    ];

    for (geometry_name, file_prefix) in &geometries {
        let geom = Geometry::build(*geometry_name)?;

        // 空の頻度データ（表示テスト用）
        let key_freq = KeyFreq::new();

        let output_path = format!("figs/{}_geometry_debug.png", file_prefix);

        render_optimized_layout(&geom, &key_freq, &output_path)?;
        println!("wrote {}", output_path);
    }

    println!("All geometry visualizations generated in figs/ directory");
    Ok(())
}
