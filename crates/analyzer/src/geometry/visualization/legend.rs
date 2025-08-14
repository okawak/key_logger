use std::io::Write;

use super::super::types::Finger;
use super::super::types::Geometry;
use super::colors::{color_of, finger_label};
use super::svg_utils::html_encode;
use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;

/// 凡例位置
#[derive(Debug, Clone, Copy)]
pub enum LegendPos {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// 絶対座標で凡例（指＋ホームのみ、タイトル無し）を描画
pub fn draw_legend_abs<W: Write>(w: &mut W, x0: f32, y0: f32) -> std::io::Result<()> {
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
            r##"<text x="{x}" y="{y}" font-family="Arial,sans-serif" font-size="11px" dominant-baseline="middle" fill="#333">{label}</text>"##,
            x = x0 + pad + sw + text_dx,
            y = y + sh * 0.5,
            label = finger_label(fgr)
        )?;
        y += row_h;
    }

    writeln!(
        w,
        r##"<text x="{x}" y="{y}" font-family="Arial,sans-serif" font-size="12px" font-weight="bold" fill="#333">Home Positions</text>"##,
        x = x0 + pad,
        y = y + sh * 0.5
    )?;
    Ok(())
}

/// コーナー位置に凡例を描画
pub fn draw_legend_corner<W: Write>(
    w: &mut W,
    pos: LegendPos,
    canvas_w: f32,
    canvas_h: f32,
    margin: f32,
    legend_width: f32,
) -> std::io::Result<()> {
    let legend_height = 220.0f32; // 推定高さ

    let (x0, y0) = match pos {
        LegendPos::TopLeft => (margin, margin),
        LegendPos::TopRight => (canvas_w - margin - legend_width, margin),
        LegendPos::BottomLeft => (margin, canvas_h - margin - legend_height),
        LegendPos::BottomRight => (
            canvas_w - margin - legend_width,
            canvas_h - margin - legend_height,
        ),
    };

    draw_legend_abs(w, x0, y0)
}

/// レイアウト凡例を描画
pub fn render_layout_legend<W: Write>(
    w: &mut W,
    geom: &Geometry,
    _freqs: &KeyFreq,
    legend_x: f32,
    legend_y: f32,
) -> Result<(), KbOptError> {
    // タイトル
    writeln!(
        w,
        r##"<text x="{}" y="{}" font-family="Arial, sans-serif" font-size="14px" font-weight="bold" fill="#000">Optimized Layout</text>"##,
        legend_x,
        legend_y + 20.0
    )?;

    // キー配置数（最適化キーのみカウント）
    let optimized_key_count = geom
        .key_placements
        .iter()
        .filter(|p| {
            matches!(
                p.placement_type,
                super::super::types::PlacementType::Optimized
                    | super::super::types::PlacementType::Arrow
            )
        })
        .count();

    writeln!(
        w,
        r##"<text x="{}" y="{}" font-family="Arial, sans-serif" font-size="12px" fill="#000">Optimized Keys: {}</text>"##,
        legend_x,
        legend_y + 50.0,
        optimized_key_count
    )?;

    // 頻度色の凡例
    writeln!(
        w,
        r##"<text x="{}" y="{}" font-family="Arial, sans-serif" font-size="12px" font-weight="bold" fill="#000">Frequency Colors:</text>"##,
        legend_x,
        legend_y + 100.0
    )?;

    let color_legends = [
        ("#ff6b6b", "≥1000"),
        ("#feca57", "≥500"),
        ("#48dbfb", "≥100"),
        ("#ddd", "&lt;100"),
    ];

    for (i, (color, label)) in color_legends.iter().enumerate() {
        let y = legend_y + 108.0 + (i as f32 * 16.0);
        writeln!(
            w,
            r##"<rect x="{}" y="{}" width="15" height="15" fill="{}" stroke="#333"/>"##,
            legend_x, y, color
        )?;
        writeln!(
            w,
            r##"<text x="{}" y="{}" font-family="Arial, sans-serif" font-size="11px" fill="#000">{}</text>"##,
            legend_x + 20.0,
            y + 12.0,
            html_encode(label)
        )?;
    }

    Ok(())
}
