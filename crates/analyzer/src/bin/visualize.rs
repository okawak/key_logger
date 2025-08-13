use analyzer::geometry::{
    DebugRenderOptions, Geometry, GeometryName, Precompute, render_svg_debug,
    visualization::layout_renderer::RenderMode,
};
use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    // figsディレクトリが存在しない場合は作成
    fs::create_dir_all("figs")?;
    let geometries = [
        (GeometryName::RowStagger, "row_stagger"),
        (GeometryName::Ortho, "ortho"),
    ];

    let opt = DebugRenderOptions {
        render_mode: RenderMode::Partition,
        legend_width_px: 320.0,
        home_offset_px: (0.0, -10.0),
        ..Default::default()
    };

    for (geometry_name, file_prefix) in &geometries {
        let geom = Geometry::build(*geometry_name)?;

        // 可視化用の空Precompute（表示には不要）
        let precomp = Precompute {
            key_cands: std::collections::HashMap::new(),
            arrow_cells: Vec::new(),
            arrow_edges: Vec::new(),
        };
        let output_path = format!("figs/{}_geometry_debug.svg", file_prefix);

        render_svg_debug(&geom, &precomp, &output_path, &opt)?;
        println!("wrote {}", output_path);
    }

    println!("All geometry visualizations generated in figs/ directory");
    Ok(())
}
