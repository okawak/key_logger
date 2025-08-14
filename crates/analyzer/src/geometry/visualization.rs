use std::fs;
use std::path::{Path, PathBuf};

use ab_glyph::{FontVec, PxScale};
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;

use super::types::*;
use crate::constants::{MARGIN, U2PX};
use crate::csv_reader::KeyFreq;
use crate::error::Result;

/// 画像描画用のコンテキスト構造体
pub struct Renderer {
    pub image: RgbImage,
    pub width: u32,
    pub height: u32,
    pub font: FontVec,
    pub render_finger_bg: bool,
}

impl Renderer {
    /// 新しいレンダラーを作成
    pub fn new(width: u32, height: u32, render_finger_bg: bool) -> Result<Self> {
        let image = ImageBuffer::from_pixel(width, height, Colors::WHITE); // 白背景

        // システムフォントを読み込み
        let font = load_system_font()?;

        Ok(Self {
            image,
            width,
            height,
            font,
            render_finger_bg,
        })
    }

    /// 矩形を描画
    pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Rgb<u8>) {
        let rect = Rect::at(x as i32, y as i32).of_size(width as u32, height as u32);
        draw_filled_rect_mut(&mut self.image, rect, color);
    }

    /// テキストを描画
    pub fn draw_text(&mut self, x: f32, y: f32, text: &str, font_size: f32, color: Rgb<u8>) {
        let scale = PxScale::from(font_size);
        draw_text_mut(
            &mut self.image,
            color,
            x as i32,
            y as i32,
            scale,
            &self.font,
            text,
        );
    }

    /// 座標変換関数を生成
    pub fn create_coord_transform(&self, y_min_u: f32) -> impl Fn(f32, f32) -> (f32, f32) + '_ {
        move |u_x: f32, u_y: f32| -> (f32, f32) {
            let px_x = MARGIN + u_x * U2PX;
            let px_y = MARGIN + (u_y - y_min_u) * U2PX;
            (px_x, px_y)
        }
    }

    /// 画像を保存
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.image.save(path)?;
        Ok(())
    }
}

/// システムフォントを読み込み
fn load_system_font() -> Result<FontVec> {
    let source = SystemSource::new();

    // Arialまたは代替フォントを探す
    let font_families = vec![
        FamilyName::Title("Arial".to_string()),
        FamilyName::SansSerif,
        FamilyName::Title("Helvetica".to_string()),
        FamilyName::Title("DejaVu Sans".to_string()),
    ];

    for family in font_families {
        if let Ok(handle) = source.select_best_match(&[family], &Properties::new())
            && let Ok(font_kit_font) = handle.load()
                // font-kitのFontからバイトデータを取得
                && let Some(font_bytes) = font_kit_font.copy_font_data()
                    && let Ok(font) = FontVec::try_from_vec(font_bytes.to_vec())
        {
            return Ok(font);
        }
    }

    // フォールバック: エラーを返す
    Err(crate::error::KbOptError::Other(
        "システムフォントが見つかりません".to_string(),
    ))
}

/// 色定義
pub struct Colors;

impl Colors {
    pub const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
    pub const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
    pub const LIGHT_GRAY: Rgb<u8> = Rgb([200, 200, 200]);
    pub const DARK_GRAY: Rgb<u8> = Rgb([128, 128, 128]);
    pub const BLUE: Rgb<u8> = Rgb([0, 100, 255]);
    pub const GREEN: Rgb<u8> = Rgb([0, 200, 0]);
    pub const RED: Rgb<u8> = Rgb([255, 0, 0]);
    pub const ORANGE: Rgb<u8> = Rgb([255, 165, 0]);
}

/// Geometryよりレイアウトを描画
pub fn render_layout<P: AsRef<Path>>(
    geom: &Geometry,
    freqs: &KeyFreq,
    output_path: P,
    render_finger_bg: bool,
) -> Result<()> {
    // 座標範囲をkey_placementsから取得（u単位で）
    let mut x_min_u = f32::INFINITY;
    let mut x_max_u = f32::NEG_INFINITY;
    let mut y_min_u = f32::INFINITY;
    let mut y_max_u = f32::NEG_INFINITY;
    
    for key_placement in geom.key_placements.values() {
        // key_placementのx, yはmm単位なので、u単位に変換
        let x_u = key_placement.x / crate::constants::U2MM as f32;
        let y_u = key_placement.y / crate::constants::U2MM as f32;
        let width_u = key_placement.width_u;
        
        x_min_u = x_min_u.min(x_u - width_u / 2.0);
        x_max_u = x_max_u.max(x_u + width_u / 2.0);
        y_min_u = y_min_u.min(y_u - 0.5); // 1u height
        y_max_u = y_max_u.max(y_u + 0.5);
    }

    let geom_w_px = (x_max_u - x_min_u + 2.0) * U2PX; // マージン含む
    let geom_h_px = (y_max_u - y_min_u + 2.0) * U2PX;
    let legend_width_px = 320.0; // 凡例エリアを拡大

    let width = (geom_w_px + legend_width_px + MARGIN * 3.0) as u32;
    let height = (geom_h_px + MARGIN * 2.0) as u32;

    // レンダラーを初期化
    let mut renderer = Renderer::new(width, height, render_finger_bg)?;

    // Geometryから統一的に描画
    render_from_geometry(&mut renderer, geom, freqs, x_min_u, x_max_u, y_min_u, y_max_u)?;

    // 凡例を描画
    render_legend(&mut renderer, geom, freqs, geom_w_px + MARGIN * 2.0, 0.0)?;

    // 画像を保存
    renderer.save(output_path)?;

    Ok(())
}

/// Geometryから統一的に描画
fn render_from_geometry(
    renderer: &mut Renderer,
    geom: &Geometry,
    freqs: &KeyFreq,
    x_min_u: f32,
    x_max_u: f32,
    y_min_u: f32,
    y_max_u: f32,
) -> Result<()> {
    let to_px = |u_x: f32, u_y: f32| -> (f32, f32) {
        let px_x = MARGIN + (u_x - x_min_u) * U2PX;
        // Y軸反転: indexが小さい方が下になるように
        let px_y = MARGIN + (y_max_u - u_y) * U2PX;
        (px_x, px_y)
    };

    // 1. 指領域（cells）を描画
    render_finger_regions(renderer, geom, &to_px)?;

    // 2. 全てのキー（key_placements）を描画
    render_all_keys(renderer, geom, freqs, &to_px)?;

    // 3. QWERTYラベルを描画（固定キーのみ）
    render_qwerty_labels_on_fixed_keys(renderer, geom, &to_px)?;

    // 4. ホームポジション（homes）を描画
    render_home_positions_from_homes(renderer, geom, &to_px)?;

    Ok(())
}

/// 指領域を描画
fn render_finger_regions(renderer: &mut Renderer, geom: &Geometry, to_px: &impl Fn(f32, f32) -> (f32, f32)) -> Result<()> {

    let cell_size_px = U2PX / 4.0; // 1cell = 0.25u

    for row in &geom.cells {
        for cell in row {
            // cell座標をu座標に変換
            let cell_x_u = cell.id.col as f32 / crate::constants::U2CELL as f32;
            let cell_y_u = cell.id.row as f32 / crate::constants::U2CELL as f32;
            let (px_x, px_y) = to_px(cell_x_u, cell_y_u);

            // 指ごとに色分け（薄い色で背景として）
            let finger_color = match cell.finger {
                Finger::LPinky => Rgb([255, 230, 230]),  // 薄い赤
                Finger::LRing => Rgb([255, 245, 230]),   // 薄いオレンジ
                Finger::LMiddle => Rgb([255, 255, 230]), // 薄い黄色
                Finger::LIndex => Rgb([230, 255, 230]),  // 薄い緑
                Finger::LThumb => Rgb([230, 230, 255]),  // 薄い青
                Finger::RThumb => Rgb([240, 230, 255]),  // 薄い紫
                Finger::RIndex => Rgb([230, 255, 230]),  // 薄い緑
                Finger::RMiddle => Rgb([255, 255, 230]), // 薄い黄色
                Finger::RRing => Rgb([255, 245, 230]),   // 薄いオレンジ
                Finger::RPinky => Rgb([255, 230, 230]),  // 薄い赤
            };

            renderer.draw_rect(px_x, px_y, cell_size_px, cell_size_px, finger_color);
        }
    }
    Ok(())
}

/// 全てのキーを描画
fn render_all_keys(
    renderer: &mut Renderer,
    geom: &Geometry,
    freqs: &KeyFreq,
    to_px: &impl Fn(f32, f32) -> (f32, f32),
) -> Result<()> {
    for (key_name, key_placement) in &geom.key_placements {
        // key_placementのx, yはmm単位なので、u単位に変換してからpx変換
        let x_u = key_placement.x / crate::constants::U2MM as f32;
        let y_u = key_placement.y / crate::constants::U2MM as f32;
        let (px_x, px_y) = to_px(x_u, y_u);
        let width_px = key_placement.width_u * U2PX;
        let height_px = U2PX; // 1u height
        
        // キー中心からキー左上角への調整
        let key_left_px = px_x - width_px / 2.0;
        let key_top_px = px_y - height_px / 2.0;
        

        // キーを黒色の四角で描画
        let key_color = Colors::BLACK;

        // キーの背景を描画
        renderer.draw_rect(key_left_px, key_top_px, width_px, height_px, key_color);

        // 境界線を描画
        let border_color = Colors::DARK_GRAY;
        let border_width = 2.0;
        renderer.draw_rect(key_left_px, key_top_px, width_px, border_width, border_color); // 上
        renderer.draw_rect(key_left_px, key_top_px, border_width, height_px, border_color); // 左
        renderer.draw_rect(
            key_left_px + width_px - border_width,
            key_top_px,
            border_width,
            height_px,
            border_color,
        ); // 右
        renderer.draw_rect(
            key_left_px,
            key_top_px + height_px - border_width,
            width_px,
            border_width,
            border_color,
        ); // 下

        // キー名を描画（キー中心）
        let text_x = px_x - 10.0;
        let text_y = px_y - 8.0;
        let text_color = Colors::WHITE; // 黒い背景に白いテキスト

        // 矢印キーの場合は記号を表示
        let display_text = if key_placement.placement_type == PlacementType::Arrow {
            match key_name.as_str() {
                "Up" => "↑",
                "Down" => "↓",
                "Left" => "←",
                "Right" => "→",
                _ => key_name.as_str(),
            }
        } else {
            key_name.as_str()
        };

        renderer.draw_text(text_x, text_y, display_text, 14.0, text_color);

        // 頻度情報を描画（最適化されたキーのみ）
        if matches!(key_placement.placement_type, PlacementType::Optimized)
            && let Some(key_id) = key_placement.key_id
        {
            let count = freqs.get_count(key_id);
            if count > 0 {
                let freq_text = format!("{}", count);
                let freq_x = key_left_px + 2.0;
                let freq_y = key_top_px + height_px - 16.0;
                renderer.draw_text(freq_x, freq_y, &freq_text, 10.0, Colors::WHITE);
            }
        }
    }
    Ok(())
}

/// QWERTYラベルを固定キーに描画
fn render_qwerty_labels_on_fixed_keys(
    renderer: &mut Renderer,
    geom: &Geometry,
    to_px: &impl Fn(f32, f32) -> (f32, f32),
) -> Result<()> {
    // QWERTY配列の定義
    let qwerty_rows = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];

    for (row_idx, row_chars) in qwerty_rows.iter().enumerate() {
        for (char_idx, ch) in row_chars.chars().enumerate() {
            let (label_x, label_y) = geom.get_qwerty_label_position(row_idx, char_idx);
            let (px_x, px_y) = to_px(label_x, label_y);

            renderer.draw_text(px_x, px_y, &ch.to_string(), 12.0, Colors::BLACK);
        }
    }
    Ok(())
}

/// ホームポジションを描画
fn render_home_positions_from_homes(
    renderer: &mut Renderer,
    geom: &Geometry,
    to_px: &impl Fn(f32, f32) -> (f32, f32),
) -> Result<()> {
    for (finger, &(home_x, home_y)) in &geom.homes {
        let (px_x, px_y) = to_px(home_x, home_y);

        // ホームポジションを小さな円として描画（矩形で近似）
        let circle_size = 8.0;
        renderer.draw_rect(
            px_x - circle_size / 2.0,
            px_y - circle_size / 2.0,
            circle_size,
            circle_size,
            Colors::RED,
        );

        // 指ラベルを描画
        let finger_label = match finger {
            Finger::LPinky => "LP",
            Finger::LRing => "LR",
            Finger::LMiddle => "LM",
            Finger::LIndex => "LI",
            Finger::LThumb => "LT",
            Finger::RThumb => "RT",
            Finger::RIndex => "RI",
            Finger::RMiddle => "RM",
            Finger::RRing => "RR",
            Finger::RPinky => "RP",
        };

        renderer.draw_text(px_x + 10.0, px_y - 8.0, finger_label, 10.0, Colors::BLACK);
    }
    Ok(())
}

/// 凡例を描画
fn render_legend(
    renderer: &mut Renderer,
    _geom: &Geometry,
    _freqs: &KeyFreq,
    legend_x: f32,
    legend_y: f32,
) -> Result<()> {
    let line_height = 20.0;
    let mut current_y = legend_y + 20.0;

    // 凡例のタイトル
    renderer.draw_text(legend_x, current_y, "Legend:", 16.0, Colors::BLACK);
    current_y += line_height * 1.5;

    // キーの説明
    renderer.draw_text(legend_x, current_y, "Keys:", 14.0, Colors::BLACK);
    current_y += line_height;

    let key_legend_items = [
        ("Fixed Keys", Colors::LIGHT_GRAY),
        ("Optimized Keys", Colors::BLUE),
        ("Arrow Keys", Colors::GREEN),
        ("Home Positions", Colors::RED),
    ];

    for (label, color) in &key_legend_items {
        // 色のサンプル矩形
        renderer.draw_rect(legend_x + 10.0, current_y, 15.0, 15.0, *color);

        // ラベル
        renderer.draw_text(legend_x + 30.0, current_y + 2.0, label, 12.0, Colors::BLACK);

        current_y += line_height;
    }

    current_y += line_height * 0.5;

    // 指領域の説明
    renderer.draw_text(legend_x, current_y, "Finger Regions:", 14.0, Colors::BLACK);
    current_y += line_height;

    let finger_legend_items = [
        ("L.Pinky", Rgb([255, 230, 230])),
        ("L.Ring", Rgb([255, 245, 230])),
        ("L.Middle", Rgb([255, 255, 230])),
        ("L.Index", Rgb([230, 255, 230])),
        ("L.Thumb", Rgb([230, 230, 255])),
        ("R.Thumb", Rgb([240, 230, 255])),
        ("R.Index", Rgb([230, 255, 230])),
        ("R.Middle", Rgb([255, 255, 230])),
        ("R.Ring", Rgb([255, 245, 230])),
        ("R.Pinky", Rgb([255, 230, 230])),
    ];

    for (label, color) in &finger_legend_items {
        // 色のサンプル矩形
        renderer.draw_rect(legend_x + 10.0, current_y, 15.0, 15.0, *color);

        // ラベル
        renderer.draw_text(legend_x + 30.0, current_y + 2.0, label, 10.0, Colors::BLACK);

        current_y += line_height * 0.8; // 少し詰める
    }

    Ok(())
}

/// figsディレクトリに最適化レイアウトを保存
pub fn save_layout(
    geom: &Geometry,
    freqs: &KeyFreq,
    render_finger_bg: bool,
    prefix: &str,
) -> Result<PathBuf> {
    let output_dir = "figs";
    fs::create_dir_all(output_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let filename = format!("{}_{:?}_{}.png", prefix, geom.name, timestamp)
        .to_lowercase()
        .replace(" ", "_");

    let output_path = Path::new(output_dir).join(&filename);

    render_layout(geom, freqs, &output_path, render_finger_bg)?;

    Ok(output_path)
}
