use analyzer::geometry::{
    DebugRenderOptions, Geometry, GeometryName, Policy, render_svg_debug,
};
use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    // figsディレクトリが存在しない場合は作成
    fs::create_dir_all("figs")?;
    let geometries = [
        (GeometryName::RowStagger, "row_stagger"),
        (GeometryName::ColStagger, "col_stagger"),
        (GeometryName::Ortho, "ortho"),
    ];

    let policy = Policy::default();
    let opt = DebugRenderOptions {
        legend_width_px: 320.0,
        home_offset_px: (0.0, -10.0),
        ..Default::default()
    };

    for (geometry_name, file_prefix) in &geometries {
        let geom = Geometry::build(*geometry_name)?;
        let output_path = format!("figs/{}_geometry_debug.svg", file_prefix);
        
        render_svg_debug(&geom, &policy, None, &output_path, &opt)?;
        println!("wrote {}", output_path);
    }

    println!("All geometry visualizations generated in figs/ directory");
    Ok(())
}
