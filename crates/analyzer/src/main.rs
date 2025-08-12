use anyhow::Result;
use std::path::PathBuf;

use analyzer::geometry::{Geometry, GeometryName};
use analyzer::optimize::{KeyFreqs, SolveOptions, solve_layout};
use analyzer::{KeyFreq, read_key_freq_from_directory, save_optimized_layout_to_figs};
use analyzer::keys::ParseOptions;

fn main() -> Result<()> {
    // 幾何
    let geom = Geometry::build(GeometryName::RowStagger)?;

    // CSVディレクトリから頻度データを読み込み
    let csv_dir = PathBuf::from("csv");
    let parse_options = ParseOptions {
        include_fkeys: false,
        fkeys_max: 12,
        include_navigation: false,
        include_numpad: false,
        strict_unknown_keys: false,
    };

    let key_freq = match read_key_freq_from_directory(&csv_dir, &parse_options) {
        Ok(freq) => {
            println!("Successfully loaded {} unique keys from {} CSV files", freq.unique_keys(), csv_dir.display());
            println!("Total key presses: {}", freq.total());
            freq
        }
        Err(e) => {
            eprintln!("Warning: Failed to read CSV files from {}: {}", csv_dir.display(), e);
            eprintln!("Using fallback test data instead.");
            
            // フォールバック: テストデータを使用
            create_fallback_data()
        }
    };

    // KeyFreqをオプティマイザー形式に変換
    let freqs: KeyFreqs = key_freq.to_optimizer_format();

    // ソルブ設定
    let opt = SolveOptions {
        include_function_keys: false,
        a_ms: 0.0, // v1：a=0
        b_ms: 1.0, // v1：b=1
        u2mm: 19.0,
        lambda_width: 0.0,
    };

    if freqs.is_empty() {
        eprintln!("Error: No key frequency data available for optimization.");
        return Ok(());
    }

    let sol = solve_layout(&geom, &freqs, &opt)?;
    println!("objective(ms): {:.3}", sol.objective_ms);
    for (k, (r, c, w)) in sol.key_place.iter() {
        println!("key {:<12} -> row {}, col {}, w {:.2}u", k, r, c, w);
    }
    for (k, bid) in sol.arrow_place.iter() {
        println!("arrow {:<12} -> row {}, bcol {}", k, bid.row, bid.bcol);
    }

    // figsディレクトリに最適化結果を画像として保存
    match save_optimized_layout_to_figs(&geom, &sol, &freqs, "rowstagger") {
        Ok(path) => println!("Optimized layout saved to: {}", path.display()),
        Err(e) => eprintln!("Failed to save layout visualization: {}", e),
    }

    Ok(())
}

/// フォールバック用のテストデータを作成
fn create_fallback_data() -> KeyFreq {
    use std::collections::HashMap;
    use analyzer::keys::KeyId;
    
    let mut counts = HashMap::new();
    
    // 数字キー
    for i in 0..=9 {
        counts.insert(KeyId::Digit(i), 100);
    }
    
    // 修飾キー
    counts.insert(KeyId::Tab, 100);
    counts.insert(KeyId::Escape, 100);
    counts.insert(KeyId::ShiftL, 100);
    counts.insert(KeyId::ShiftR, 100);
    counts.insert(KeyId::CtrlL, 100);
    counts.insert(KeyId::CtrlR, 100);
    counts.insert(KeyId::AltL, 100);
    counts.insert(KeyId::AltR, 100);
    counts.insert(KeyId::MetaL, 100);
    counts.insert(KeyId::MetaR, 100);
    counts.insert(KeyId::CapsLock, 100);
    counts.insert(KeyId::Delete, 100);
    counts.insert(KeyId::Backspace, 100);
    
    // 矢印キー
    counts.insert(KeyId::Arrow(analyzer::keys::ArrowKey::Up), 1000);
    counts.insert(KeyId::Arrow(analyzer::keys::ArrowKey::Down), 1000);
    counts.insert(KeyId::Arrow(analyzer::keys::ArrowKey::Left), 1000);
    counts.insert(KeyId::Arrow(analyzer::keys::ArrowKey::Right), 1000);
    
    KeyFreq::from_counts(counts)
}
