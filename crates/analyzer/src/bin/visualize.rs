use analyzer::csv_reader::KeyFreq;
use analyzer::geometry::{Geometry, GeometryName, save_layout};
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

        let output_path = save_layout(&geom, &key_freq, true, file_prefix)?;
        info!("wrote {}", output_path.display());
    }

    info!("All geometry visualizations generated in figs/ directory");
    Ok(())
}
