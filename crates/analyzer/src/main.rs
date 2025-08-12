use anyhow::Result;
use std::collections::HashMap;

use analyzer::geometry::{Geometry, GeometryName};
use analyzer::optimize::{KeyFreqs, SolveOptions, solve_layout};

fn main() -> Result<()> {
    // 幾何
    let geom = Geometry::build(GeometryName::RowStagger)?;

    // 例：頻度（実際はCSVから）
    let mut freqs: KeyFreqs = HashMap::new();
    // 通常キー
    for k in [
        "Tab",
        "Escape",
        "LeftShift",
        "RightShift",
        "LeftControl",
        "RightControl",
        "LeftAlt",
        "RightAlt",
        "LeftMeta",
        "RightMeta",
        "CapsLock",
        "Delete",
        "Backspace",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "0",
    ] {
        freqs.insert(k.to_string(), 100);
    }
    // 矢印
    freqs.insert("ArrowUp".into(), 1000);
    freqs.insert("ArrowDown".into(), 1000);
    freqs.insert("ArrowLeft".into(), 1000);
    freqs.insert("ArrowRight".into(), 1000);

    // ソルブ設定
    let opt = SolveOptions {
        include_function_keys: false,
        a_ms: 0.0, // v1：a=0
        b_ms: 1.0, // v1：b=1
        u2mm: 19.0,
        lambda_width: 0.0,
    };

    let sol = solve_layout(&geom, &freqs, &opt)?;
    println!("objective(ms): {:.3}", sol.objective_ms);
    for (k, (r, c, w)) in sol.key_place.iter() {
        println!("key {:<12} -> row {}, col {}, w {:.2}u", k, r, c, w);
    }
    for (k, bid) in sol.arrow_place.iter() {
        println!("arrow {:<12} -> row {}, bcol {}", k, bid.row, bid.bcol);
    }

    Ok(())
}
