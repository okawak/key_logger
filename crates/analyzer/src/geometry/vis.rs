use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use super::policy::Policy;
use super::precompute::Precompute;
use super::types::*;
use crate::error::KbOptError;

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
            show_legend: true,
            legend_pos: LegendPos::TopRight,
            legend_outside: true,
            legend_width_px: 280.0,
            home_offset_px: (0.0, -8.0),
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
