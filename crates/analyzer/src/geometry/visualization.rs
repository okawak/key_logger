use crate::{
    config::Config,
    constants::{FONT_SIZE, LEGEND_WIDTH, MARGIN, MAX_COL_CELLS, MAX_ROW, U2CELL, U2MM, U2PX},
    csv_reader::KeyFreq,
    error::{KbOptError, Result},
    geometry::types::*,
};

use ab_glyph::{FontVec, PxScale};
use font_kit::{family_name::FamilyName, properties::Properties, source::SystemSource};
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_text_mut},
    rect::Rect,
};
use std::{fs, path::Path};

/// キー中心座標をピクセル座標に変換（Y軸反転、center-to-center）
#[inline]
fn key_center_to_px(u_x: f64, u_y: f64) -> (f64, f64) {
    let px_x = MARGIN + u_x * U2PX;
    let px_y = MARGIN + (MAX_ROW as f64 - u_y) * U2PX;
    (px_x, px_y)
}

/// Cell中心座標をピクセル座標に変換（Y軸反転、cell-to-center）
#[inline]
fn cell_center_to_px(cell_row: usize, cell_col: usize) -> (f64, f64) {
    let u_x = (cell_col as f64 + 0.5) / U2CELL as f64;
    let u_y = cell_row as f64 + 0.5; // 行は既にu単位なので、中心計算のため0.5を加算
    key_center_to_px(u_x, u_y)
}

/// 画像描画用のコンテキスト構造体
pub struct Renderer {
    pub image: RgbImage,
    pub width: u32,
    pub height: u32,
    pub font: FontVec,
}

impl Renderer {
    /// 新しいレンダラーを作成
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let image = ImageBuffer::from_pixel(width, height, Colors::WHITE); // 白背景

        // システムフォントを読み込み
        let font = load_system_font()?;

        Ok(Self {
            image,
            width,
            height,
            font,
        })
    }

    /// 矩形を描画（塗りつぶし）
    pub fn draw_rect(&mut self, x: f64, y: f64, width: f64, height: f64, color: Rgb<u8>) {
        let rect = Rect::at(x as i32, y as i32).of_size(width as u32, height as u32);
        draw_filled_rect_mut(&mut self.image, rect, color);
    }

    /// 矩形の境界線を描画
    pub fn draw_rect_outline(&mut self, x: f64, y: f64, width: f64, height: f64, color: Rgb<u8>) {
        let rect = Rect::at(x as i32, y as i32).of_size(width as u32, height as u32);
        draw_hollow_rect_mut(&mut self.image, rect, color);
    }

    /// テキストを描画
    pub fn draw_text(&mut self, x: f64, y: f64, text: &str, font_size: f64, color: Rgb<u8>) {
        let scale = PxScale::from(font_size as f32);
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
    pub fn create_coord_transform(&self, y_min_u: f64) -> impl Fn(f64, f64) -> (f64, f64) + '_ {
        move |u_x: f64, u_y: f64| -> (f64, f64) {
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
        FamilyName::Title("Firge35Nerd Console".to_string()),
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
    Err(KbOptError::Other(
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
    pub const LIGHT_BLUE: Rgb<u8> = Rgb([121, 166, 242]);
    pub const GREEN: Rgb<u8> = Rgb([0, 200, 0]);
    pub const LIGHT_GREEN: Rgb<u8> = Rgb([154, 230, 113]);
    pub const LIGHT_YELLOW: Rgb<u8> = Rgb([255, 255, 128]); // 数字キー用の色
    pub const RED: Rgb<u8> = Rgb([255, 0, 0]);
    pub const ORANGE: Rgb<u8> = Rgb([255, 165, 0]);
    pub const LIGHT_PURPLE: Rgb<u8> = Rgb([200, 150, 255]); // レイヤキー用の色
    pub const LIGHT_ORANGE: Rgb<u8> = Rgb([255, 200, 150]); // モディファイアキー用の色
    // 指領域の色（薄い色で背景として）
    pub const BG_RED: Rgb<u8> = Rgb([255, 230, 230]); // 薄い赤
    pub const BG_ORANGE: Rgb<u8> = Rgb([255, 245, 230]); // 薄いオレンジ
    pub const BG_YELLOW: Rgb<u8> = Rgb([255, 255, 230]); // 薄い黄色
    pub const BG_GREEN: Rgb<u8> = Rgb([230, 255, 230]); // 薄い緑
    pub const BG_BLUE: Rgb<u8> = Rgb([230, 230, 255]); // 薄い青
    pub const BG_PURPLE: Rgb<u8> = Rgb([250, 230, 255]); // 薄い紫
    pub const BG_GREEN2: Rgb<u8> = Rgb([210, 255, 240]); // 薄い緑
}

/// Geometryよりレイアウトを描画
pub fn render_layout<P: AsRef<Path>>(
    geom: &Geometry,
    freqs: &KeyFreq,
    output_path: P,
    render_finger_bg: bool,
) -> Result<()> {
    let geom_w_px = (MAX_COL_CELLS as f64 / U2CELL as f64) * U2PX;
    let geom_h_px = MAX_ROW as f64 * U2PX;

    let width = (geom_w_px + LEGEND_WIDTH + MARGIN * 3.0) as u32; // 左、中央、右のマージン
    let height = (geom_h_px + MARGIN * 2.0) as u32; // 上下のマージン

    // レンダラーを初期化
    let mut renderer = Renderer::new(width, height)?;

    // Geometryから統一的に描画
    render_from_geometry(&mut renderer, geom, freqs, render_finger_bg)?;

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
    render_finger_bg: bool,
) -> Result<()> {
    // 1. 指領域（cells）を描画
    if render_finger_bg {
        render_finger_regions(renderer, geom)?;
    }

    // 2. 全てのキー（key_placements）を描画（ラベル含む）
    render_all_keys(renderer, geom, freqs)?;

    // 3. ホームポジション（homes）を描画
    render_home_positions_from_homes(renderer, geom)?;

    Ok(())
}

/// 指領域を描画
fn render_finger_regions(renderer: &mut Renderer, geom: &Geometry) -> Result<()> {
    let cell_size_px = U2PX / U2CELL as f64; // 1cell -> px

    for row in &geom.cells {
        for cell in row {
            // cell中心座標をピクセル座標に変換
            let (px_x, px_y) = cell_center_to_px(cell.id.row, cell.id.col);

            // 指ごとに色分け（薄い色で背景として）
            let finger_color = match cell.finger {
                Finger::LPinky => Colors::BG_RED,
                Finger::LRing => Colors::BG_ORANGE,
                Finger::LMiddle => Colors::BG_YELLOW,
                Finger::LIndex => Colors::BG_GREEN,
                Finger::LThumb => Colors::BG_BLUE,
                Finger::RThumb => Colors::BG_PURPLE,
                Finger::RIndex => Colors::BG_GREEN2,
                Finger::RMiddle => Colors::BG_YELLOW,
                Finger::RRing => Colors::BG_ORANGE,
                Finger::RPinky => Colors::BG_RED,
            };

            // cell中心から左上角に調整（キーと同じ方式）
            let cell_left_px = px_x - cell_size_px / 2.0;
            let cell_top_px = px_y - U2PX / 2.0;
            renderer.draw_rect(cell_left_px, cell_top_px, cell_size_px, U2PX, finger_color);
        }
    }
    Ok(())
}

/// 全てのキーを描画
fn render_all_keys(renderer: &mut Renderer, geom: &Geometry, freqs: &KeyFreq) -> Result<()> {
    for (key_name, key_placement) in &geom.key_placements {
        // key_placementのx, yはmm単位なので、u単位に変換してからpx変換
        let x_u = key_placement.x / U2MM;
        let y_u = key_placement.y / U2MM;
        let (px_x, px_y) = key_center_to_px(x_u, y_u);

        let width_px = key_placement.width_u * U2PX;
        let height_px = U2PX; // 1u height

        // キー中心からキー左上角への調整
        let key_left_px = px_x - width_px / 2.0;
        let key_top_px = px_y - height_px / 2.0;

        // キータイプに応じて描画方法を変更
        match key_placement.placement_type {
            PlacementType::Fixed => {
                // 固定キーは黒枠のみ
                renderer.draw_rect_outline(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::BLACK,
                );
            }
            PlacementType::Optimized => {
                // 最適化キーは青い塗りつぶし
                renderer.draw_rect(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::LIGHT_BLUE,
                );
                renderer.draw_rect_outline(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::BLACK,
                );
            }
            PlacementType::Arrow => {
                // 矢印キーは緑の塗りつぶし
                renderer.draw_rect(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::LIGHT_GREEN,
                );
                renderer.draw_rect_outline(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::BLACK,
                );
            }
        }

        // 記号を表示
        let display_text = match key_name.as_str() {
            "ArrowUp" => "↑",
            "ArrowDown" => "↓",
            "ArrowLeft" => "←",
            "ArrowRight" => "→",
            "Backslash" => r"\",
            "Slash" => "/",
            "RBracket" => "]",
            "LBracket" => "[",
            "Semicolon" => ";",
            "Equal" => "=",
            "Minus" => "-",
            "Backtick" => "`",
            "Quote" => "'",
            "RightShift" => "R⇧",
            "Period" => ".",
            "Comma" => ",",
            "LeftShift" => "L⇧",
            "Space" => "△",
            "LeftControl" => "LCtrl",
            "RightControl" => "RCtrl",
            "LeftAlt" => "LAlt",
            "RightAlt" => "RAlt",
            "LeftMeta" => "LMeta",
            "RightMeta" => "RMeta",
            "Backspace" => "BS",
            "Delete" => "Del",
            "CapsLock" => "Caps",
            "Escape" => "Esc",
            "Tab" => "Tab",
            "Enter" => "Enter",
            // KeyIdのDebug形式に対応
            s if s.starts_with("Digit(") => {
                // "Digit(3)" -> "3"
                s.trim_start_matches("Digit(").trim_end_matches(")")
            }
            s if s.starts_with("Letter(") => {
                // "Letter(A)" -> "A"
                s.trim_start_matches("Letter(").trim_end_matches(")")
            }
            s if s.starts_with("Symbol(") => {
                // "Symbol(Comma)" -> "," など、個別マッピングが必要
                match s {
                    "Symbol(Comma)" => ",",
                    "Symbol(Period)" => ".",
                    "Symbol(Slash)" => "/",
                    "Symbol(Semicolon)" => ";",
                    "Symbol(Quote)" => "'",
                    "Symbol(LBracket)" => "[",
                    "Symbol(RBracket)" => "]",
                    "Symbol(Backslash)" => r"\",
                    "Symbol(Backtick)" => "`",
                    "Symbol(Minus)" => "-",
                    "Symbol(Equal)" => "=",
                    _ => s,
                }
            }
            s if s.starts_with("Arrow(") => {
                // "Arrow(Up)" -> "↑"
                match s {
                    "Arrow(Up)" => "↑",
                    "Arrow(Down)" => "↓",
                    "Arrow(Left)" => "←",
                    "Arrow(Right)" => "→",
                    _ => s,
                }
            }
            _ => key_name.as_str(),
        };

        // キー名を描画（キー中心）
        let text_x = px_x - U2PX / 10.0 - U2PX / 15.0 * (display_text.chars().count() - 1) as f64;
        let text_y = px_y - U2PX / 3.0; // offsetを調整
        let text_color = Colors::BLACK; // 透明背景に黒いテキスト

        renderer.draw_text(text_x, text_y, display_text, FONT_SIZE, text_color);

        // 頻度情報を描画
        if let Some(key_id) = key_placement.key_id {
            let count = freqs.get_count(key_id);
            if count > 0 {
                let freq_text = format!("{}", count);
                let freq_x = key_left_px + 2.0;
                let freq_y = key_top_px + height_px - 16.0;
                renderer.draw_text(freq_x, freq_y, &freq_text, 10.0, Colors::BLACK);
            }
        }
    }
    Ok(())
}

/// ホームポジションを描画
fn render_home_positions_from_homes(renderer: &mut Renderer, geom: &Geometry) -> Result<()> {
    for (home_x, home_y) in geom.homes.values() {
        // home座標はmm単位なので、u単位に変換してからpx変換
        let x_u = home_x / U2MM;
        let y_u = home_y / U2MM;
        let (px_x, px_y) = key_center_to_px(x_u, y_u);

        // ホームポジションを小さな円として描画（矩形で近似）
        let circle_size = 8.0;
        renderer.draw_rect(
            px_x - circle_size / 2.0,
            px_y - circle_size / 2.0,
            circle_size,
            circle_size,
            Colors::RED,
        );
    }
    Ok(())
}

/// 凡例を描画
fn render_legend(
    renderer: &mut Renderer,
    _geom: &Geometry,
    _freqs: &KeyFreq,
    legend_x: f64,
    legend_y: f64,
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

/// 指定されたレイヤのGeometryを描画
#[allow(dead_code)]
fn render_layer_geometry(
    renderer: &mut Renderer,
    geom: &Geometry,
    freqs: &KeyFreq,
    render_finger_bg: bool,
    y_offset: f64,
    layer_symbols: &[(String, usize, String)], // (symbol, layer_number, modifier_key)
) -> Result<()> {
    // 1. 指領域（cells）を描画
    if render_finger_bg {
        render_finger_regions_with_offset(renderer, geom, y_offset)?;
    }

    // 2. 全てのキー（key_placements）を描画（ラベル含む）
    render_all_keys_with_offset(renderer, geom, freqs, y_offset)?;

    // 3. レイヤ特有の記号を描画
    render_layer_symbols(renderer, geom, layer_symbols, y_offset)?;

    // 4. ホームポジション（homes）を描画
    render_home_positions_with_offset(renderer, geom, y_offset)?;

    Ok(())
}

/// オフセット付きで指領域を描画
fn render_finger_regions_with_offset(
    renderer: &mut Renderer,
    geom: &Geometry,
    y_offset: f64,
) -> Result<()> {
    let cell_size_px = U2PX / U2CELL as f64; // 1cell -> px

    for row in &geom.cells {
        for cell in row {
            // cell中心座標をピクセル座標に変換
            let (px_x, px_y) = cell_center_to_px(cell.id.row, cell.id.col);
            let adjusted_px_y = px_y + y_offset;

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

            // cell中心から左上角に調整（キーと同じ方式）
            let cell_left_px = px_x - cell_size_px / 2.0;
            let cell_top_px = adjusted_px_y - U2PX / 2.0;
            renderer.draw_rect(cell_left_px, cell_top_px, cell_size_px, U2PX, finger_color);
        }
    }
    Ok(())
}

/// オフセット付きで全てのキーを描画
fn render_all_keys_with_offset(
    renderer: &mut Renderer,
    geom: &Geometry,
    freqs: &KeyFreq,
    y_offset: f64,
) -> Result<()> {
    for (key_name, key_placement) in &geom.key_placements {
        // key_placementのx, yはmm単位なので、u単位に変換してからpx変換
        let x_u = key_placement.x / U2MM;
        let y_u = key_placement.y / U2MM;
        let (px_x, px_y) = key_center_to_px(x_u, y_u);
        let adjusted_px_y = px_y + y_offset;

        let width_px = key_placement.width_u * U2PX;
        let height_px = U2PX; // 1u height

        // キー中心からキー左上角への調整
        let key_left_px = px_x - width_px / 2.0;
        let key_top_px = adjusted_px_y - height_px / 2.0;

        // キータイプに応じて描画方法を変更
        match key_placement.placement_type {
            PlacementType::Fixed => {
                // 固定キーは黒枠のみ
                renderer.draw_rect_outline(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::BLACK,
                );
            }
            PlacementType::Optimized => {
                // 最適化キーは青い塗りつぶし
                renderer.draw_rect(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::LIGHT_BLUE,
                );
                renderer.draw_rect_outline(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::BLACK,
                );
            }
            PlacementType::Arrow => {
                // 矢印キーは緑の塗りつぶし
                renderer.draw_rect(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::LIGHT_GREEN,
                );
                renderer.draw_rect_outline(
                    key_left_px,
                    key_top_px,
                    width_px,
                    height_px,
                    Colors::BLACK,
                );
            }
        }

        // 記号を表示
        let display_text = match key_name.as_str() {
            "ArrowUp" => "↑",
            "ArrowDown" => "↓",
            "ArrowLeft" => "←",
            "ArrowRight" => "→",
            "Backslash" => r"\",
            "Slash" => "/",
            "RBracket" => "]",
            "LBracket" => "[",
            "Semicolon" => ";",
            "Equal" => "=",
            "Minus" => "-",
            "Backtick" => "`",
            "Quote" => "'",
            "RightShift" => "R⇧",
            "Period" => ".",
            "Comma" => ",",
            "LeftShift" => "L⇧",
            "Space" => "△",
            "LeftControl" => "LCtrl",
            "RightControl" => "RCtrl",
            "LeftAlt" => "LAlt",
            "RightAlt" => "RAlt",
            "LeftMeta" => "LMeta",
            "RightMeta" => "RMeta",
            "Backspace" => "BS",
            "Delete" => "Del",
            "CapsLock" => "Caps",
            "Escape" => "Esc",
            "Tab" => "Tab",
            "Enter" => "Enter",
            // 数字キーの対応を追加
            "Digit(0)" => "0",
            "Digit(1)" => "1",
            "Digit(2)" => "2",
            "Digit(3)" => "3",
            "Digit(4)" => "4",
            "Digit(5)" => "5",
            "Digit(6)" => "6",
            "Digit(7)" => "7",
            "Digit(8)" => "8",
            "Digit(9)" => "9",
            // KeyIdのDebug形式に対応
            s if s.starts_with("Digit(") => {
                // "Digit(3)" -> "3"
                s.trim_start_matches("Digit(").trim_end_matches(")")
            }
            s if s.starts_with("Symbol(") => {
                // "Symbol(Comma)" -> "," など、個別マッピングが必要
                match s {
                    "Symbol(Comma)" => ",",
                    "Symbol(Period)" => ".",
                    "Symbol(Slash)" => "/",
                    "Symbol(Semicolon)" => ";",
                    "Symbol(Quote)" => "'",
                    "Symbol(LBracket)" => "[",
                    "Symbol(RBracket)" => "]",
                    "Symbol(Backslash)" => r"\",
                    "Symbol(Backtick)" => "`",
                    "Symbol(Minus)" => "-",
                    "Symbol(Equal)" => "=",
                    _ => s,
                }
            }
            s if s.starts_with("Arrow(") => {
                // "Arrow(Up)" -> "↑"
                match s {
                    "Arrow(Up)" => "↑",
                    "Arrow(Down)" => "↓",
                    "Arrow(Left)" => "←",
                    "Arrow(Right)" => "→",
                    _ => s,
                }
            }
            _ => key_name.as_str(),
        };

        // キー名を描画（キー中心）
        let text_x = px_x - U2PX / 10.0 - U2PX / 15.0 * (display_text.chars().count() - 1) as f64;
        let text_y = adjusted_px_y - U2PX / 3.0; // offsetを調整
        let text_color = Colors::BLACK; // 透明背景に黒いテキスト

        renderer.draw_text(text_x, text_y, display_text, FONT_SIZE, text_color);

        // 頻度情報を描画
        if let Some(key_id) = key_placement.key_id {
            let count = freqs.get_count(key_id);
            if count > 0 {
                let freq_text = format!("{}", count);
                let freq_x = key_left_px + 2.0;
                let freq_y = key_top_px + height_px - 16.0;
                renderer.draw_text(freq_x, freq_y, &freq_text, 10.0, Colors::BLACK);
            }
        }
    }
    Ok(())
}

/// レイヤ記号を描画（アルファベットキーの上に重ねて表示）
#[allow(dead_code)]
fn render_layer_symbols(
    renderer: &mut Renderer,
    geom: &Geometry,
    layer_symbols: &[(String, usize, String)], // (symbol, layer_number, modifier_key)
    y_offset: f64,
) -> Result<()> {
    // アルファベットキーの座標を取得して、その上に記号を表示
    for (symbol, _layer_num, modifier_key) in layer_symbols {
        // アルファベットキーの位置を検索（例：'a'キーなど）
        if let Some(placement) = geom.key_placements.values().find(|p| {
            // アルファベットキーの典型的な配置を検索
            p.placement_type == PlacementType::Optimized
        }) {
            let x_u = placement.x / U2MM;
            let y_u = placement.y / U2MM;
            let (px_x, px_y) = key_center_to_px(x_u, y_u);
            let adjusted_px_y = px_y + y_offset;

            let width_px = placement.width_u * U2PX;
            let height_px = U2PX;

            // レイヤ記号用の背景を描画（半透明効果）
            let key_left_px = px_x - width_px / 2.0;
            let key_top_px = adjusted_px_y - height_px / 2.0;

            renderer.draw_rect(
                key_left_px,
                key_top_px,
                width_px,
                height_px / 3.0, // 上部1/3に描画
                Colors::LIGHT_PURPLE,
            );

            // レイヤ記号を描画
            let text_x = px_x - U2PX / 15.0 * (symbol.chars().count() - 1) as f64;
            let text_y = adjusted_px_y - U2PX / 2.0 + 5.0; // キーの上部
            renderer.draw_text(text_x, text_y, symbol, FONT_SIZE * 0.8, Colors::BLACK);

            // モディファイアキーの表示
            let mod_text = format!("({}+)", modifier_key);
            let mod_x = key_left_px + 2.0;
            let mod_y = key_top_px + height_px - 12.0;
            renderer.draw_text(mod_x, mod_y, &mod_text, 8.0, Colors::DARK_GRAY);
        }
    }
    Ok(())
}

/// オフセット付きでホームポジションを描画
fn render_home_positions_with_offset(
    renderer: &mut Renderer,
    geom: &Geometry,
    y_offset: f64,
) -> Result<()> {
    for (home_x, home_y) in geom.homes.values() {
        // home座標はmm単位なので、u単位に変換してからpx変換
        let x_u = home_x / U2MM;
        let y_u = home_y / U2MM;
        let (px_x, px_y) = key_center_to_px(x_u, y_u);
        let adjusted_px_y = px_y + y_offset;

        // ホームポジションを小さな円として描画（矩形で近似）
        let circle_size = 8.0;
        renderer.draw_rect(
            px_x - circle_size / 2.0,
            adjusted_px_y - circle_size / 2.0,
            circle_size,
            circle_size,
            Colors::RED,
        );
    }
    Ok(())
}

/// figsディレクトリに最適化レイアウトを保存
pub fn save_layout(
    geom: &Geometry,
    freqs: Option<&KeyFreq>,
    config: &Config,
    render_finger_bg: bool,
    prefix: &str,
) -> Result<()> {
    let output_dir = &config.solver.output_dir;
    fs::create_dir_all(output_dir)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let filename = format!("{}_{:?}_{}.png", prefix, geom.name, timestamp)
        .to_lowercase()
        .replace(" ", "_");

    let output_path = Path::new(output_dir).join(&filename);

    let freq_data = if let Some(f) = freqs {
        f
    } else {
        &KeyFreq::new() // 空の頻度データ
    };

    render_layout(geom, freq_data, &output_path, render_finger_bg)?;

    Ok(())
}
