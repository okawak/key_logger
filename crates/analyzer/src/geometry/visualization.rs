// 可視化モジュール - 公開API

pub mod colors;
pub mod components;
pub mod layout_renderer;
pub mod legend;
pub mod svg_utils;

// 公開APIの再エクスポート
pub use layout_renderer::{
    DebugRenderOptions, RenderMode, render_optimized_layout, render_svg_debug,
};
pub use legend::LegendPos;
pub use svg_utils::{save_optimized_layout, save_optimized_layout_to_figs};

// 便利な型エイリアス
pub use layout_renderer::DebugRenderOptions as RenderOptions;
