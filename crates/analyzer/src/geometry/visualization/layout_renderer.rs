use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use super::super::policy::Policy;
use super::super::precompute::Precompute;
use super::super::types::*;
use super::components::*;
use super::legend::{LegendPos, draw_legend_corner, render_layout_legend};
use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;
use crate::optimize::SolutionLayout;

/// 描画モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// F(j) による 0.25u セルのパーティション塗り（非重畳・推奨）
    Partition,
    /// 参考: 指境界の縦スラブ（オーバーレイ用途）
    ZonesVertical,
    /// 最適化結果レイアウト表示
    OptimizedLayout,
}

/// デバッグ描画オプション
#[derive(Debug, Clone)]
pub struct DebugRenderOptions {
    pub scale_px_per_u: f32,
    pub margin_px: f32,

    /// 描画モード
    pub render_mode: RenderMode,

    /// セル描画（Partitionモード）
    pub show_partition_cells: bool, // Partition のときのみ使用
    pub show_partition_borders: bool, // 隣接指が異なる辺に 1px 線
    pub show_fixed_letters: bool,
    pub show_qwerty_labels: bool,
    pub show_homes: bool,

    /// OptimizedLayoutモード用
    pub show_optimized_keys: bool, // 最適化されたキー配置を表示
    pub show_key_labels: bool,      // キーラベルを表示
    pub show_key_frequencies: bool, // キー頻度を表示
    pub show_arrow_keys: bool,      // 矢印キーを表示

    /// 凡例
    pub show_legend: bool,
    pub legend_pos: LegendPos,
    pub legend_outside: bool,
    pub legend_width_px: f32,

    /// ホームポジション描画
    pub home_offset_px: (f32, f32),
}

impl Default for DebugRenderOptions {
    fn default() -> Self {
        Self {
            scale_px_per_u: 60.0,
            margin_px: 24.0,
            render_mode: RenderMode::Partition,
            show_partition_cells: true,
            show_partition_borders: true,
            show_fixed_letters: false,
            show_qwerty_labels: false,
            show_homes: true,
            show_optimized_keys: false,
            show_key_labels: false,
            show_key_frequencies: false,
            show_arrow_keys: false,
            show_legend: true,
            legend_pos: LegendPos::TopRight,
            legend_outside: true,
            legend_width_px: 240.0,
            home_offset_px: (0.0, -6.0),
        }
    }
}

impl DebugRenderOptions {
    /// 最適化レイアウト表示用のオプション
    pub fn for_optimized_layout() -> Self {
        Self {
            scale_px_per_u: 60.0,
            margin_px: 24.0,
            render_mode: RenderMode::OptimizedLayout,
            show_partition_cells: false,
            show_partition_borders: false,
            show_fixed_letters: true,
            show_qwerty_labels: true,
            show_homes: true,
            show_optimized_keys: true,
            show_key_labels: true,
            show_key_frequencies: true,
            show_arrow_keys: true,
            show_legend: true,
            legend_pos: LegendPos::TopRight,
            legend_outside: false,
            legend_width_px: 270.0,
            home_offset_px: (0.0, -6.0),
        }
    }
}

/// デバッグ用SVG描画
pub fn render_svg_debug<P: AsRef<Path>>(
    geom: &Geometry,
    _precomp: &Precompute,
    _policy: &Policy,
    output_path: P,
    opt: &DebugRenderOptions,
) -> Result<(), KbOptError> {
    let file = File::create(output_path)?;
    let mut f = BufWriter::new(file);

    // Y軸の範囲を取得
    let mut y_min_u = f32::INFINITY;
    let mut y_max_u = f32::NEG_INFINITY;
    for row in &geom.cells {
        for cell in row {
            y_min_u = y_min_u.min(cell.center_y_u);
            y_max_u = y_max_u.max(cell.center_y_u);
        }
    }

    // キャンバス計算
    let s = opt.scale_px_per_u;
    let m = opt.margin_px;
    let geom_w_px = 15.0 * s;
    let geom_h_px = (y_max_u - y_min_u + 1.0) * s;

    let (w, h) = if opt.legend_outside && opt.show_legend {
        match opt.legend_pos {
            LegendPos::TopRight | LegendPos::BottomRight => (
                (geom_w_px + opt.legend_width_px + m * 3.0) as u32,
                (geom_h_px + m * 2.0) as u32,
            ),
            _ => ((geom_w_px + m * 2.0) as u32, (geom_h_px + m * 2.0) as u32),
        }
    } else {
        ((geom_w_px + m * 2.0) as u32, (geom_h_px + m * 2.0) as u32)
    };

    // SVGヘッダー
    writeln!(
        f,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"##,
        w, h, w, h
    )?;
    writeln!(
        f,
        r##"<rect x="0" y="0" width="{}" height="{}" fill="white"/>"##,
        w, h
    )?;

    // 座標変換関数
    let to_px = |u_x: f32, u_y: f32| -> (f32, f32) {
        let px_x = m + u_x * s;
        let px_y = m + (u_y - y_min_u) * s;
        (px_x, px_y)
    };

    // 描画モード別処理
    match opt.render_mode {
        RenderMode::Partition => {
            render_partition_mode(&mut f, geom, opt, &to_px)?;
        }
        RenderMode::ZonesVertical => {
            render_zones_vertical_mode(&mut f, geom, opt, &to_px)?;
        }
        RenderMode::OptimizedLayout => {
            // OptimizedLayoutモードは専用の関数で処理
            return Err(KbOptError::Other(
                "OptimizedLayoutモードはrender_optimized_layout()を使用してください".to_string(),
            ));
        }
    }

    // 凡例描画
    if opt.show_legend && opt.legend_outside {
        draw_legend_corner(
            &mut f,
            opt.legend_pos,
            w as f32,
            h as f32,
            m,
            opt.legend_width_px,
        )?;
    }

    writeln!(f, "</svg>")?;
    f.flush()?;

    Ok(())
}

/// 最適化レイアウトの描画
pub fn render_optimized_layout<P: AsRef<Path>>(
    geom: &Geometry,
    solution: &SolutionLayout,
    freqs: &KeyFreq,
    output_path: P,
) -> Result<(), KbOptError> {
    let opt = DebugRenderOptions::for_optimized_layout();
    let file = File::create(output_path)?;
    let mut f = BufWriter::new(file);

    // Y軸の範囲を取得
    let mut y_min_u = f32::INFINITY;
    let mut y_max_u = f32::NEG_INFINITY;
    for row in &geom.cells {
        for cell in row {
            y_min_u = y_min_u.min(cell.center_y_u);
            y_max_u = y_max_u.max(cell.center_y_u);
        }
    }

    // キャンバス計算
    let s = opt.scale_px_per_u;
    let m = opt.margin_px;
    let geom_w_px = 15.0 * s;
    let geom_h_px = (y_max_u - y_min_u + 1.0) * s;
    let legend_width = if opt.show_legend {
        opt.legend_width_px
    } else {
        0.0
    };
    let w = (geom_w_px + legend_width + m * 3.0) as u32;
    let h = (geom_h_px + m * 2.0) as u32;

    // SVGヘッダー
    writeln!(
        f,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"##,
        w, h, w, h
    )?;
    writeln!(
        f,
        r##"<rect x="0" y="0" width="{}" height="{}" fill="white"/>"##,
        w, h
    )?;

    // 座標変換関数
    let to_px = |u_x: f32, u_y: f32| -> (f32, f32) {
        let px_x = m + u_x * s;
        let px_y = m + (u_y - y_min_u) * s;
        (px_x, px_y)
    };

    // 固定文字（アルファベット）の枠を描画
    if opt.show_fixed_letters {
        render_fixed_letters(&mut f, geom, &opt, &to_px)?;
    }

    // QWERTYラベルを描画
    if opt.show_qwerty_labels {
        render_qwerty_labels(&mut f, geom, &opt, &to_px)?;
    }

    // 最適化されたキーを描画
    if opt.show_optimized_keys {
        render_optimized_keys(&mut f, geom, solution, freqs, &opt, &to_px)?;
    }

    // 矢印キーを描画
    if opt.show_arrow_keys {
        render_arrow_keys(&mut f, geom, solution, freqs, &opt, &to_px)?;
    }

    // ホームポジションを描画
    if opt.show_homes {
        render_home_positions(&mut f, geom, &opt, &to_px)?;
    }

    // 凡例を描画
    if opt.show_legend {
        render_layout_legend(&mut f, solution, freqs, geom_w_px + m * 2.0, 0.0)?;
    }

    writeln!(f, "</svg>")?;
    f.flush()?;

    Ok(())
}

// プライベートヘルパー関数
fn render_partition_mode<W: Write>(
    _w: &mut W,
    _geom: &Geometry,
    _opt: &DebugRenderOptions,
    _to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    // Partitionモードの実装は元のコードから移植が必要
    // ここでは簡略化
    Ok(())
}

fn render_zones_vertical_mode<W: Write>(
    _w: &mut W,
    _geom: &Geometry,
    _opt: &DebugRenderOptions,
    _to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    // ZonesVerticalモードの実装は元のコードから移植が必要
    // ここでは簡略化
    Ok(())
}
