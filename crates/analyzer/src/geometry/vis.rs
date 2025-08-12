use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use super::policy::Policy;
use super::precompute::Precompute;
use super::types::*;
use crate::error::KbOptError;
use crate::optimize::{SolutionLayout, KeyFreqs};

#[derive(Debug, Clone, Copy)]
pub enum LegendPos {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

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

#[derive(Debug, Clone)]
pub struct DebugRenderOptions {
    pub scale_px_per_u: f32,
    pub margin_px: f32,

    // モード（既定: パーティション）
    pub render_mode: RenderMode,

    // レイヤ切替
    pub show_partition_cells: bool,   // Partition のときのみ使用
    pub show_partition_borders: bool, // 隣接指が異なる辺に 1px 線
    pub show_fixed_letters: bool,
    pub show_qwerty_labels: bool,
    pub show_homes: bool,

    // 最適化結果用オプション
    pub show_optimized_keys: bool,     // 最適化されたキー配置を表示
    pub show_key_labels: bool,         // キーラベルを表示
    pub show_key_frequencies: bool,    // キー頻度を表示
    pub show_arrow_keys: bool,         // 矢印キーを表示

    // 凡例
    pub show_legend: bool,
    pub legend_pos: LegendPos,
    pub legend_outside: bool,
    pub legend_width_px: f32,

    // ホーム●のピクセルオフセット
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
            show_fixed_letters: true,
            show_qwerty_labels: true,
            show_homes: true,
            // 最適化結果用のデフォルト
            show_optimized_keys: true,
            show_key_labels: true,
            show_key_frequencies: false,
            show_arrow_keys: true,
            show_legend: true,
            legend_pos: LegendPos::TopRight,
            legend_outside: true,
            legend_width_px: 280.0,
            home_offset_px: (0.0, -8.0),
        }
    }
}

/// 最適化結果表示用のオプションを作成
impl DebugRenderOptions {
    pub fn for_optimized_layout() -> Self {
        Self {
            render_mode: RenderMode::OptimizedLayout,
            show_partition_cells: false,
            show_partition_borders: false,
            show_fixed_letters: true,  // 固定キー（アルファベット）の枠を表示
            show_qwerty_labels: true,  // QWERTYラベルを表示
            show_homes: true,
            show_optimized_keys: true,
            show_key_labels: true,
            show_key_frequencies: true,
            show_arrow_keys: true,
            ..Default::default()
        }
    }
}

/// 左右で色を変える（左右で系統を分ける）
fn color_of(fgr: Finger) -> &'static str {
    match fgr {
        Finger::LPinky => "#ff9aa2",
        Finger::LRing => "#ffbfa3",
        Finger::LMiddle => "#fff4a3",
        Finger::LIndex => "#b9ffb7",
        Finger::LThumb => "#b5d6ff",
        Finger::RThumb => "#98c7ff",
        Finger::RIndex => "#a7fff0",
        Finger::RMiddle => "#fff08a",
        Finger::RRing => "#ffc89b",
        Finger::RPinky => "#ff8c94",
    }
}
fn finger_label(fgr: Finger) -> &'static str {
    match fgr {
        Finger::LPinky => "Left Pinky",
        Finger::LRing => "Left Ring",
        Finger::LMiddle => "Left Middle",
        Finger::LIndex => "Left Index",
        Finger::LThumb => "Left Thumb",
        Finger::RThumb => "Right Thumb",
        Finger::RIndex => "Right Index",
        Finger::RMiddle => "Right Middle",
        Finger::RRing => "Right Ring",
        Finger::RPinky => "Right Pinky",
    }
}

/// F(j)パーティション（非重畳）＋必要に応じて枠・ラベル・ホーム・凡例
pub fn render_svg_debug<P: AsRef<Path>>(
    geom: &Geometry,
    _policy: &Policy,
    _pre: Option<&Precompute>,
    out_path: P,
    opt: &DebugRenderOptions,
) -> Result<(), KbOptError> {
    let s = opt.scale_px_per_u;
    let m = opt.margin_px;

    let width_u = 15.0f32;
    let y_min_u = geom
        .cfg
        .rows
        .iter()
        .map(|r| r.base_y_u - 0.5)
        .fold(f32::INFINITY, f32::min);
    let y_max_u = geom
        .cfg
        .rows
        .iter()
        .map(|r| r.base_y_u + 0.5)
        .fold(f32::NEG_INFINITY, f32::max);
    let height_u = (y_max_u - y_min_u).max(5.5);

    let legend_extra_w = if opt.show_legend && opt.legend_outside {
        opt.legend_width_px
    } else {
        0.0
    };
    let w = (width_u * s + 2.0 * m + legend_extra_w).ceil() as i32;
    let h = (height_u * s + 2.0 * m).ceil() as i32;

    let mut f = BufWriter::new(File::create(out_path)?);

    writeln!(
        f,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"##,
        w = w,
        h = h
    )?;
    writeln!(
        f,
        r##"<rect x="0" y="0" width="{w}" height="{h}" fill="white"/>"##,
        w = w,
        h = h
    )?;

    let sx = |x_u: f32| -> f32 { m + x_u * s };
    let sy = |y_u: f32| -> f32 { m + (y_u - y_min_u) * s };

    // 盤面クリップ
    writeln!(
        f,
        r##"<defs><clipPath id="kbdClip"><rect x="{x}" y="{y}" width="{w}" height="{h}"/></clipPath></defs>"##,
        x = sx(0.0),
        y = sy(y_min_u),
        w = s * width_u,
        h = s * (y_max_u - y_min_u)
    )?;

    /* -------------------- 1) F(j) パーティション塗り -------------------- */
    if opt.render_mode == RenderMode::Partition && opt.show_partition_cells {
        writeln!(f, r##"<g clip-path="url(#kbdClip)">"##)?;

        // 全体のセルグリッドを描画（0.0から15.0uまで）
        let total_cells_x = (width_u / CELL_U) as usize; // 60セル
        for r in 0..geom.cfg.rows.len() {
            let row = &geom.cfg.rows[r];
            for c_grid in 0..total_cells_x {
                let x0 = c_grid as f32 * CELL_U;
                let y0 = row.base_y_u;

                // セル中心のx座標を計算
                let cell_center_x = x0 + 0.5 * CELL_U;

                // このグリッド位置がgeom.cellsに対応するかチェック
                let finger = if x0 >= row.offset_u
                    && (x0 - row.offset_u) / CELL_U < geom.cells_per_row as f32
                {
                    let c_in_row = ((x0 - row.offset_u) / CELL_U) as usize;
                    if c_in_row < geom.cells[r].len() {
                        geom.cells[r][c_in_row].finger
                    } else {
                        // geom.cellsの範囲外でもF(j)を使用
                        if r == geom.cfg.thumb_row {
                            if cell_center_x < 7.5 {
                                Finger::LThumb
                            } else {
                                Finger::RThumb
                            }
                        } else {
                            finger_from_x(cell_center_x, &geom.cfg.finger_x_boundaries)
                        }
                    }
                } else {
                    // 左側のパディング部分もF(j)を使用
                    if r == geom.cfg.thumb_row {
                        if cell_center_x < 7.5 {
                            Finger::LThumb
                        } else {
                            Finger::RThumb
                        }
                    } else {
                        finger_from_x(cell_center_x, &geom.cfg.finger_x_boundaries)
                    }
                };
                let fill = color_of(finger);

                writeln!(
                    f,
                    r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}" fill-opacity="0.9" stroke="none"/>"##,
                    x = sx(x0),
                    y = sy(y0 - 0.5),
                    w = s * CELL_U,
                    h = s * ONE_U,
                    fill = fill
                )?;
            }
        }
        writeln!(f, r##"</g>"##)?;
    }

    /* --------------- 2) 固定文字（1u枠） --------------- */
    if opt.show_fixed_letters {
        writeln!(f, r##"<g clip-path="url(#kbdClip)">"##)?;
        for row_idx in 0..geom.cfg.rows.len() {
            let mut c = 0usize;
            while c < geom.cells_per_row {
                if !geom.cells[row_idx][c].fixed_occupied {
                    c += 1;
                    continue;
                }
                let start = c;
                while c < geom.cells_per_row && geom.cells[row_idx][c].fixed_occupied {
                    c += 1;
                }
                let mut cc = start;
                while cc < c {
                    let (x0, y0) = geom.get_fixed_key_position(row_idx, cc);

                    writeln!(
                        f,
                        r##"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="none" stroke="#222" stroke-width="1.2"/>"##,
                        x = sx(x0),
                        y = sy(y0),
                        w = s * ONE_U,
                        h = s * 1.0
                    )?;
                    cc += cells_from_u(ONE_U);
                }
            }
        }
        writeln!(f, r##"</g>"##)?;
    }

    /* --------------- 3) QWERTY ラベル --------------- */
    if opt.show_qwerty_labels {
        let mut label = |txt: &str, x: f32, y: f32| -> std::io::Result<()> {
            writeln!(
                f,
                r##"<text x="{x}" y="{y}" font-family="monospace" font-size="{fs}" text-anchor="middle" dominant-baseline="central" fill="#333">{txt}</text>"##,
                x = sx(x),
                y = sy(y),
                fs = 12.0
            )
        };
        let row1 = 1usize;
        for (i, ch) in "QWERTYUIOP".chars().enumerate() {
            let (x, y) = geom.get_qwerty_label_position(row1, i);
            let _ = label(&ch.to_string(), x, y);
        }
        let row2 = 2usize;
        for (i, ch) in "ASDFGHJKL".chars().enumerate() {
            let (x, y) = geom.get_qwerty_label_position(row2, i);
            let _ = label(&ch.to_string(), x, y);
        }
        let row3 = 3usize;
        for (i, ch) in "ZXCVBNM".chars().enumerate() {
            let (x, y) = geom.get_qwerty_label_position(row3, i);
            let _ = label(&ch.to_string(), x, y);
        }
    }

    /* --------------- 4) ホーム位置 --------------- */
    if opt.show_homes {
        let (dx, dy) = opt.home_offset_px;
        for (hx, hy) in geom.homes.values() {
            writeln!(
                f,
                r##"<circle cx="{x}" cy="{y}" r="3.5" fill="#333" fill-opacity="0.9"/>"##,
                x = sx(*hx) + dx,
                y = sy(*hy) + dy
            )?;
        }
    }

    /* --------------- 5) 凡例 --------------- */
    if opt.show_legend {
        if opt.legend_outside {
            let items = 11.0f32;
            let pad = 10.0f32;
            let row_h = 18.0f32;
            let box_h = pad * 2.0 + row_h * items;
            let legend_x = (m + width_u * s + 10.0).ceil();
            let legend_y_top = (m).ceil();
            let legend_y_bottom = (h as f32 - box_h - 10.0).max(m).ceil();
            let legend_y = match opt.legend_pos {
                LegendPos::TopLeft | LegendPos::TopRight => legend_y_top,
                LegendPos::BottomLeft | LegendPos::BottomRight => legend_y_bottom,
            };
            draw_legend_abs(&mut f, legend_x, legend_y)?;
        } else {
            draw_legend_corner(&mut f, w as f32, h as f32, opt)?;
        }
    }

    writeln!(f, r##"</svg>"##)?;
    Ok(())
}

/* ---------- 凡例（指＋ホームのみ、タイトル無し） ---------- */

fn draw_legend_abs<W: Write>(w: &mut W, x0: f32, y0: f32) -> std::io::Result<()> {
    let pad = 10.0f32;
    let row_h = 18.0f32;
    let sw = 12.0f32;
    let sh = 12.0f32;
    let text_dx = 6.0f32;
    let items = 11usize;
    let box_w = 240.0f32;
    let box_h = pad * 2.0 + row_h * items as f32;
    writeln!(
        w,
        r##"<rect x="{x}" y="{y}" width="{bw}" height="{bh}" rx="8" ry="8" fill="#fff" stroke="#aaa" stroke-width="1"/>"##,
        x = x0,
        y = y0,
        bw = box_w,
        bh = box_h
    )?;
    let mut y = y0 + pad;
    let fingers = [
        Finger::LPinky,
        Finger::LRing,
        Finger::LMiddle,
        Finger::LIndex,
        Finger::LThumb,
        Finger::RThumb,
        Finger::RIndex,
        Finger::RMiddle,
        Finger::RRing,
        Finger::RPinky,
    ];
    for fgr in fingers {
        let fill = color_of(fgr);
        writeln!(
            w,
            r##"<rect x="{x}" y="{y}" width="{sw}" height="{sh}" fill="{fill}" stroke="#666" stroke-width="0.5"/>"##,
            x = x0 + pad,
            y = y,
            sw = sw,
            sh = sh,
            fill = fill
        )?;
        writeln!(
            w,
            r##"<text x="{x}" y="{y}" font-family="sans-serif" font-size="12" fill="#333" dominant-baseline="hanging">{label}</text>"##,
            x = x0 + pad + sw + text_dx,
            y = y,
            label = finger_label(fgr)
        )?;
        y += row_h;
    }
    writeln!(
        w,
        r##"<circle cx="{x}" cy="{y}" r="4" fill="#333"/>"##,
        x = x0 + pad + 6.0,
        y = y + 6.0
    )?;
    writeln!(
        w,
        r##"<text x="{x}" y="{y}" font-family="sans-serif" font-size="12" fill="#333" dominant-baseline="hanging">Home position</text>"##,
        x = x0 + pad + sw + text_dx,
        y = y
    )?;
    Ok(())
}
fn draw_legend_corner<W: Write>(
    w: &mut W,
    canvas_w: f32,
    canvas_h: f32,
    opt: &DebugRenderOptions,
) -> std::io::Result<()> {
    let pad = 10.0f32;
    let row_h = 18.0f32;
    let items = 11usize;
    let box_w = 240.0f32;
    let box_h = pad * 2.0 + row_h * items as f32;
    let (x0, y0) = match opt.legend_pos {
        LegendPos::TopLeft => (10.0, 10.0),
        LegendPos::TopRight => (canvas_w - box_w - 10.0, 10.0),
        LegendPos::BottomLeft => (10.0, canvas_h - box_h - 10.0),
        LegendPos::BottomRight => (canvas_w - box_w - 10.0, canvas_h - box_h - 10.0),
    };
    draw_legend_abs(w, x0, y0)
}

/// 最適化結果のキーボード配列をSVGとして描画
pub fn render_optimized_layout<P: AsRef<Path>>(
    geom: &Geometry,
    solution: &SolutionLayout,
    freqs: &KeyFreqs,
    out_path: P,
    opt: &DebugRenderOptions,
) -> Result<(), KbOptError> {
    let s = opt.scale_px_per_u;
    let m = opt.margin_px;

    let width_u = 15.0f32;
    let y_min_u = geom
        .cfg
        .rows
        .iter()
        .map(|r| r.base_y_u - 0.5)
        .fold(f32::INFINITY, f32::min);
    let y_max_u = geom
        .cfg
        .rows
        .iter()
        .map(|r| r.base_y_u + 0.5)
        .fold(f32::NEG_INFINITY, f32::max);
    let height_u = (y_max_u - y_min_u).max(5.5);

    let legend_extra_w = if opt.show_legend && opt.legend_outside {
        opt.legend_width_px
    } else {
        0.0
    };
    let w = (width_u * s + 2.0 * m + legend_extra_w).ceil() as i32;
    let h = (height_u * s + 2.0 * m).ceil() as i32;

    let mut f = BufWriter::new(File::create(out_path)?);

    writeln!(
        f,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"##,
        w = w,
        h = h
    )?;
    writeln!(
        f,
        r##"<rect x="0" y="0" width="{w}" height="{h}" fill="white"/>"##,
        w = w,
        h = h
    )?;

    // 座標系の変換のためのヘルパー
    // SVGでは上から下にY座標が増加するため、base_y_uが小さい行を上に表示
    let to_px = |u_x: f32, u_y: f32| -> (f32, f32) {
        let px_x = m + u_x * s;
        let px_y = m + (u_y - y_min_u) * s;
        (px_x, px_y)
    };

    // 固定文字（アルファベット）の枠を描画
    if opt.show_fixed_letters {
        render_fixed_letters(&mut f, geom, opt, &to_px)?;
    }

    // QWERTYラベルを描画
    if opt.show_qwerty_labels {
        render_qwerty_labels(&mut f, geom, opt, &to_px)?;
    }

    // 最適化されたキーを描画
    if opt.show_optimized_keys {
        render_optimized_keys(&mut f, geom, solution, freqs, opt, &to_px)?;
    }

    // 矢印キーを描画
    if opt.show_arrow_keys {
        render_arrow_keys(&mut f, geom, solution, freqs, opt, &to_px)?;
    }

    // ホームポジションを描画
    if opt.show_homes {
        render_home_positions(&mut f, geom, opt, &to_px)?;
    }

    // 凡例を描画
    if opt.show_legend {
        render_layout_legend(&mut f, solution, freqs, opt, w as f32, h as f32)?;
    }

    writeln!(f, "</svg>")?;
    f.flush()?;

    Ok(())
}

/// 最適化されたキーの描画
fn render_optimized_keys<W: Write>(
    w: &mut W,
    geom: &Geometry,
    solution: &SolutionLayout,
    freqs: &KeyFreqs,
    opt: &DebugRenderOptions,
    to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    for (key_name, &(row, start_col, width_u)) in &solution.key_place {
        // キーの位置とサイズを計算（固定キーと同じ座標系を使用）
        let row_config = &geom.cfg.rows[row];
        let x_start_u = row_config.offset_u + (start_col as f32) * CELL_U;
        let y_u = row_config.base_y_u - 0.5; // 固定キーと同じY座標計算
        let width_px = width_u * opt.scale_px_per_u;
        let height_px = 1.0 * opt.scale_px_per_u; // 1u height

        let (px_x, px_y) = to_px(x_start_u, y_u + 0.5);

        // キーの背景色（頻度に基づく色分けまたは指の色）
        let key_color = get_key_color(key_name, freqs);
        
        // キーの矩形を描画
        writeln!(
            w,
            r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" stroke="#333" stroke-width="1"/>"##,
            px_x, px_y - height_px * 0.5, width_px, height_px, key_color
        )?;

        // キーラベルを描画
        if opt.show_key_labels {
            let label_x = px_x + width_px * 0.5;
            let label_y = px_y;
            let font_size = (12.0 * opt.scale_px_per_u / 60.0).max(8.0).min(16.0);
            let encoded_key_name = html_encode(key_name);
            
            writeln!(
                w,
                r##"<text x="{:.2}" y="{:.2}" font-family="Arial, sans-serif" font-size="{:.1}px" text-anchor="middle" dominant-baseline="middle" fill="#000">{}</text>"##,
                label_x, label_y, font_size, encoded_key_name
            )?;

            // 頻度を表示
            if opt.show_key_frequencies {
                if let Some(&freq) = freqs.get(key_name) {
                    let freq_y = label_y + font_size * 0.8;
                    writeln!(
                        w,
                        r##"<text x="{:.2}" y="{:.2}" font-family="Arial, sans-serif" font-size="{:.1}px" text-anchor="middle" dominant-baseline="middle" fill="#666">{}</text>"##,
                        label_x, freq_y, font_size * 0.7, freq
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// 矢印キーの描画
fn render_arrow_keys<W: Write>(
    w: &mut W,
    geom: &Geometry,
    solution: &SolutionLayout,
    freqs: &KeyFreqs,
    opt: &DebugRenderOptions,
    to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    for (arrow_name, block_id) in &solution.arrow_place {
        // ブロック位置を計算（固定キーと同じ座標系を使用）
        let row_config = &geom.cfg.rows[block_id.row];
        let block_x_u = row_config.offset_u + (block_id.bcol * 4) as f32 * CELL_U; // 1u = 4 cells
        let block_y_u = row_config.base_y_u - 0.5; // 固定キーと同じY座標計算
        let block_size_px = opt.scale_px_per_u; // 1u square

        let (px_x, px_y) = to_px(block_x_u, block_y_u + 0.5);

        // 矢印キーの背景色
        let arrow_color = "#e0e0e0";
        
        // 矢印キーの矩形を描画
        writeln!(
            w,
            r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" stroke="#333" stroke-width="2"/>"##,
            px_x, px_y - block_size_px * 0.5, block_size_px, block_size_px, arrow_color
        )?;

        // 矢印シンボルを描画
        if opt.show_key_labels {
            let symbol = match arrow_name.as_str() {
                "ArrowUp" => "↑",
                "ArrowDown" => "↓",
                "ArrowLeft" => "←",
                "ArrowRight" => "→",
                _ => "?",
            };
            
            let label_x = px_x + block_size_px * 0.5;
            let label_y = px_y;
            let font_size = (16.0 * opt.scale_px_per_u / 60.0).max(10.0).min(20.0);
            
            writeln!(
                w,
                r##"<text x="{:.2}" y="{:.2}" font-family="Arial, sans-serif" font-size="{:.1}px" text-anchor="middle" dominant-baseline="middle" fill="#000">{}</text>"##,
                label_x, label_y, font_size, symbol
            )?;

            // 頻度を表示
            if opt.show_key_frequencies {
                if let Some(&freq) = freqs.get(arrow_name) {
                    let freq_y = label_y + font_size * 0.8;
                    writeln!(
                        w,
                        r##"<text x="{:.2}" y="{:.2}" font-family="Arial, sans-serif" font-size="{:.1}px" text-anchor="middle" dominant-baseline="middle" fill="#666">{}</text>"##,
                        label_x, freq_y, font_size * 0.6, freq
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// ホームポジションの描画
fn render_home_positions<W: Write>(
    w: &mut W,
    geom: &Geometry,
    opt: &DebugRenderOptions,
    to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    for (&finger, &(home_x, home_y)) in &geom.homes {
        let (px_x, px_y) = to_px(home_x, home_y);
        let (offset_x, offset_y) = opt.home_offset_px;
        let adjusted_x = px_x + offset_x;
        let adjusted_y = px_y + offset_y;

        let color = color_of(finger);
        let radius = 4.0;

        writeln!(
            w,
            r##"<circle cx="{:.2}" cy="{:.2}" r="{:.1}" fill="{}" stroke="#000" stroke-width="1"/>"##,
            adjusted_x, adjusted_y, radius, color
        )?;
    }
    Ok(())
}

/// キーの色を決定（頻度または固定色）
fn get_key_color(key_name: &str, freqs: &KeyFreqs) -> &'static str {
    // 矢印キーの場合
    if key_name.starts_with("Arrow") {
        return "#e0e0e0";
    }
    
    // 頻度に基づく色分け（簡単な例）
    if let Some(&freq) = freqs.get(key_name) {
        if freq >= 1000 {
            "#ff6b6b" // 高頻度: 赤
        } else if freq >= 500 {
            "#feca57" // 中高頻度: オレンジ
        } else if freq >= 100 {
            "#48dbfb" // 中頻度: 青
        } else {
            "#ddd" // 低頻度: グレー
        }
    } else {
        "#f0f0f0" // デフォルト: ライトグレー
    }
}

/// レイアウト用の凡例を描画
fn render_layout_legend<W: Write>(
    w: &mut W,
    solution: &SolutionLayout,
    _freqs: &KeyFreqs,
    opt: &DebugRenderOptions,
    canvas_w: f32,
    _canvas_h: f32,
) -> Result<(), KbOptError> {
    if !opt.legend_outside {
        return Ok(()); // 内部凡例は実装せず
    }

    let legend_x = canvas_w - opt.legend_width_px + 10.0;
    let legend_y = 20.0;
    let line_height = 20.0;
    let mut y_offset = 0.0;

    // 凡例のタイトル
    writeln!(
        w,
        r##"<text x="{:.1}" y="{:.1}" font-family="Arial, sans-serif" font-size="14px" font-weight="bold" fill="#000">Optimized Layout</text>"##,
        legend_x, legend_y + y_offset
    )?;
    y_offset += line_height * 1.5;

    // 統計情報
    writeln!(
        w,
        r##"<text x="{:.1}" y="{:.1}" font-family="Arial, sans-serif" font-size="12px" fill="#000">Objective: {:.1}ms</text>"##,
        legend_x, legend_y + y_offset, solution.objective_ms
    )?;
    y_offset += line_height;

    let total_keys = solution.key_place.len() + solution.arrow_place.len();
    writeln!(
        w,
        r##"<text x="{:.1}" y="{:.1}" font-family="Arial, sans-serif" font-size="12px" fill="#000">Total Keys: {}</text>"##,
        legend_x, legend_y + y_offset, total_keys
    )?;
    y_offset += line_height * 1.5;

    // 頻度による色分けの説明
    writeln!(
        w,
        r##"<text x="{:.1}" y="{:.1}" font-family="Arial, sans-serif" font-size="12px" font-weight="bold" fill="#000">Frequency Colors:</text>"##,
        legend_x, legend_y + y_offset
    )?;
    y_offset += line_height;

    let freq_ranges = [
        ("≥1000", "#ff6b6b"),
        ("≥500", "#feca57"),
        ("≥100", "#48dbfb"),
        ("&lt;100", "#ddd"), // HTMLエンコードされた<
    ];

    for (label, color) in freq_ranges {
        writeln!(
            w,
            r##"<rect x="{:.1}" y="{:.1}" width="15" height="15" fill="{}" stroke="#333"/>"##,
            legend_x, legend_y + y_offset - 12.0, color
        )?;
        writeln!(
            w,
            r##"<text x="{:.1}" y="{:.1}" font-family="Arial, sans-serif" font-size="11px" fill="#000">{}</text>"##,
            legend_x + 20.0, legend_y + y_offset, label
        )?;
        y_offset += line_height * 0.8;
    }

    Ok(())
}

/// 最適化結果をfigsディレクトリに自動保存する便利関数
pub fn save_optimized_layout_to_figs(
    geom: &Geometry,
    solution: &SolutionLayout,
    freqs: &KeyFreqs,
    geometry_name: &str,
) -> Result<PathBuf, KbOptError> {
    // figsディレクトリを作成
    let figs_dir = PathBuf::from("figs");
    fs::create_dir_all(&figs_dir).map_err(|e| KbOptError::Io(e))?;

    // ファイル名を生成（ジオメトリ名 + タイムスタンプ）
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("optimized_{}_{}.svg", geometry_name.to_lowercase(), timestamp);
    let output_path = figs_dir.join(&filename);

    // 最適化結果用のレンダリングオプション
    let render_opts = DebugRenderOptions::for_optimized_layout();

    // SVG画像を生成
    render_optimized_layout(geom, solution, freqs, &output_path, &render_opts)?;

    Ok(output_path)
}

/// 最適化結果を指定パスに保存する関数（より汎用的）
pub fn save_optimized_layout<P: AsRef<Path>>(
    geom: &Geometry,
    solution: &SolutionLayout,
    freqs: &KeyFreqs,
    output_path: P,
    options: Option<&DebugRenderOptions>,
) -> Result<(), KbOptError> {
    // 出力ディレクトリを作成
    if let Some(parent_dir) = output_path.as_ref().parent() {
        fs::create_dir_all(parent_dir).map_err(|e| KbOptError::Io(e))?;
    }

    let default_opts = DebugRenderOptions::for_optimized_layout();
    let render_opts = options.unwrap_or(&default_opts);
    render_optimized_layout(geom, solution, freqs, output_path, render_opts)
}

/// 固定文字（アルファベット）の枠を描画
fn render_fixed_letters<W: Write>(
    w: &mut W,
    geom: &Geometry,
    opt: &DebugRenderOptions,
    to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    for row_idx in 0..geom.cfg.rows.len() {
        let mut c = 0usize;
        while c < geom.cells_per_row {
            if !geom.cells[row_idx][c].fixed_occupied {
                c += 1;
                continue;
            }
            let start = c;
            while c < geom.cells_per_row && geom.cells[row_idx][c].fixed_occupied {
                c += 1;
            }
            let mut cc = start;
            while cc < c {
                let (x0_u, y0_u) = geom.get_fixed_key_position(row_idx, cc);
                let (px_x, px_y) = to_px(x0_u, y0_u + 0.5);
                let size_px = opt.scale_px_per_u;

                writeln!(
                    w,
                    r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="none" stroke="#222" stroke-width="1.2"/>"##,
                    px_x,
                    px_y - size_px * 0.5,
                    size_px,
                    size_px
                )?;
                cc += cells_from_u(ONE_U);
            }
        }
    }
    Ok(())
}

/// QWERTYラベルを描画
fn render_qwerty_labels<W: Write>(
    w: &mut W,
    geom: &Geometry,
    opt: &DebugRenderOptions,
    to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    let font_size = (12.0 * opt.scale_px_per_u / 60.0).max(8.0).min(16.0);
    
    let mut draw_label = |text: &str, x_u: f32, y_u: f32| -> std::io::Result<()> {
        let (px_x, px_y) = to_px(x_u, y_u);
        let encoded_text = html_encode(text);
        writeln!(
            w,
            r##"<text x="{:.2}" y="{:.2}" font-family="Arial, sans-serif" font-size="{:.1}px" text-anchor="middle" dominant-baseline="middle" fill="#333">{}</text>"##,
            px_x, px_y, font_size, encoded_text
        )
    };

    // Row 1: QWERTYUIOP
    let row1 = 1usize;
    for (i, ch) in "QWERTYUIOP".chars().enumerate() {
        let (x, y) = geom.get_qwerty_label_position(row1, i);
        let _ = draw_label(&ch.to_string(), x, y);
    }

    // Row 2: ASDFGHJKL
    let row2 = 2usize;
    for (i, ch) in "ASDFGHJKL".chars().enumerate() {
        let (x, y) = geom.get_qwerty_label_position(row2, i);
        let _ = draw_label(&ch.to_string(), x, y);
    }

    // Row 3: ZXCVBNM
    let row3 = 3usize;
    for (i, ch) in "ZXCVBNM".chars().enumerate() {
        let (x, y) = geom.get_qwerty_label_position(row3, i);
        let _ = draw_label(&ch.to_string(), x, y);
    }

    Ok(())
}

/// HTMLエンティティをエンコード（SVG内でのテキスト表示用）
fn html_encode(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&apos;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
