use std::io::Write;

use super::super::types::PlacementType;
use super::super::types::*;
use super::colors::{color_of, get_key_color};
use super::svg_utils::html_encode;
use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;

// Constants for cell calculations
const CELL_U: f32 = 0.25; // Each cell is 0.25u

/// ホームポジションの描画
pub fn render_home_positions<W: Write>(
    w: &mut W,
    geom: &Geometry,
    opt: &super::layout_renderer::DebugRenderOptions,
    to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    for (finger, &(home_x, home_y)) in &geom.homes {
        let (px_x, px_y) = to_px(home_x, home_y);
        let (offset_x, offset_y) = opt.home_offset_px;
        let circle_x = px_x + offset_x;
        let circle_y = px_y + offset_y;
        let color = color_of(*finger);

        writeln!(
            w,
            r##"<circle cx="{:.2}" cy="{:.2}" r="4.0" fill="{}" stroke="#000" stroke-width="1"/>"##,
            circle_x, circle_y, color
        )?;
    }
    Ok(())
}

/// 固定文字（アルファベット）の枠を描画
pub fn render_fixed_letters<W: Write>(
    w: &mut W,
    geom: &Geometry,
    opt: &super::layout_renderer::DebugRenderOptions,
    to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    // アルファベット文字の配置定義（行インデックス, 文字数）
    // 新しい行インデックス: 0=親指, 1=ZXCV, 2=ASDF, 3=QWERTY, 4=数字
    let letter_layouts = [
        (3, 10), // Row 3: QWERTYUIOP (10 keys)
        (2, 9),  // Row 2: ASDFGHJKL (9 keys)
        (1, 7),  // Row 1: ZXCVBNM (7 keys)
    ];

    for (row_idx, key_count) in letter_layouts {
        for char_idx in 0..key_count {
            let (x0_u, y0_u) = geom.get_fixed_key_position(row_idx, char_idx);
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
        }
    }
    Ok(())
}

/// QWERTYラベルを描画
pub fn render_qwerty_labels<W: Write>(
    w: &mut W,
    geom: &Geometry,
    _opt: &super::DebugRenderOptions,
    to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    let qwerty_layouts = [
        (
            3,
            ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"].as_slice(),
        ),
        (2, ["A", "S", "D", "F", "G", "H", "J", "K", "L"].as_slice()),
        (1, ["Z", "X", "C", "V", "B", "N", "M"].as_slice()),
    ];

    for (row_idx, chars) in qwerty_layouts {
        for (char_idx, &ch) in chars.iter().enumerate() {
            let (label_x, label_y) = geom.get_qwerty_label_position(row_idx, char_idx);
            let (px_x, px_y) = to_px(label_x, label_y);

            writeln!(
                w,
                r##"<text x="{:.2}" y="{:.2}" font-family="Arial, sans-serif" font-size="12.0px" text-anchor="middle" dominant-baseline="middle" fill="#333">{}</text>"##,
                px_x, px_y, ch
            )?;
        }
    }
    Ok(())
}

/// Geometryから全キー配置を描画（固定キー、最適化キー、矢印キーを統一的に処理）
pub fn render_all_keys_from_geometry<W: Write>(
    w: &mut W,
    geom: &Geometry,
    freqs: &KeyFreq,
    opt: &super::layout_renderer::DebugRenderOptions,
    to_px: &dyn Fn(f32, f32) -> (f32, f32),
) -> Result<(), KbOptError> {
    // 全キー配置を統一的に描画
    for key_placement in &geom.key_placements {
        let x_start_u = (key_placement.start_col as f32) * CELL_U;
        let y_u = key_placement.row as f32 - 0.5;
        let width_px = key_placement.width_u * opt.scale_px_per_u;
        let height_px = 1.0 * opt.scale_px_per_u;

        let (px_x, px_y) = to_px(x_start_u, y_u + 0.5);

        // キーの背景色とストローク（キータイプ別に設定）
        let (key_color, stroke_width) = match key_placement.placement_type {
            PlacementType::Fixed => ("#f0f0f0", "1"), // 淡いグレー
            PlacementType::Optimized => (get_key_color(&key_placement.key_name, freqs), "1"), // 頻度ベース
            PlacementType::Arrow => ("#e0e0e0", "2"), // 矢印キー用の色とストローク
        };

        // キーの矩形を描画
        writeln!(
            w,
            r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="{}" stroke="#333" stroke-width="{}"/>"##,
            px_x,
            px_y - height_px * 0.5,
            width_px,
            height_px,
            key_color,
            stroke_width
        )?;

        // ラベルの描画（矢印キーは特別な記号を使用）
        if opt.show_key_labels {
            let label_x = px_x + width_px * 0.5;
            let label_y = px_y;

            let (label_text, font_size) = match key_placement.placement_type {
                PlacementType::Arrow => {
                    // 矢印キー用の記号
                    let symbol = match key_placement.key_name.as_str() {
                        "ArrowUp" => "↑",
                        "ArrowDown" => "↓",
                        "ArrowLeft" => "←",
                        "ArrowRight" => "→",
                        _ => "?",
                    };
                    (symbol.to_string(), "16.0px")
                }
                _ => (html_encode(&key_placement.key_name), "12.0px"),
            };

            writeln!(
                w,
                r##"<text x="{:.2}" y="{:.2}" font-family="Arial, sans-serif" font-size="{}" text-anchor="middle" dominant-baseline="middle" fill="#000">{}</text>"##,
                label_x, label_y, font_size, label_text
            )?;
        }

        // 頻度を描画（最適化キーと矢印キーのみ）
        if opt.show_key_frequencies
            && matches!(
                key_placement.placement_type,
                PlacementType::Optimized | PlacementType::Arrow
            )
        {
            let freq_x = px_x + width_px * 0.5;
            let freq_y = px_y
                + match key_placement.placement_type {
                    PlacementType::Arrow => height_px * 0.28, // 矢印キー用のオフセット
                    _ => 9.6,                                 // 通常キー用のオフセット
                };

            // key_nameから頻度を取得
            let freq = freqs
                .counts()
                .iter()
                .find(|(k, _)| k.to_string() == key_placement.key_name)
                .map(|(_, &count)| count)
                .unwrap_or(0);

            let font_size = match key_placement.placement_type {
                PlacementType::Arrow => "9.6px",
                _ => "8.4px",
            };

            writeln!(
                w,
                r##"<text x="{:.2}" y="{:.2}" font-family="Arial, sans-serif" font-size="{}" text-anchor="middle" dominant-baseline="middle" fill="#666">{}</text>"##,
                freq_x, freq_y, font_size, freq
            )?;
        }
    }

    Ok(())
}
