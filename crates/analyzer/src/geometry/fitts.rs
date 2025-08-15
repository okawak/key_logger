use super::types::*;
use crate::constants::U2CELL;

/// Fitts' law (v1: W_eff = w)
#[inline]
pub fn fitts_time(a: f64, b: f64, dist_u: f64, width_u: f64) -> f64 {
    a + b * ((dist_u / width_u) + 1.0).log2()
}

impl Geometry {
    /// Distance from finger home to key center [u]
    pub fn distance_u(&self, start: CellId, width_u: f32) -> f64 {
        let row = start.row;
        let center_col = start.col + ((width_u * U2CELL as f32).round() as usize) / 2;
        let x = center_col as f32 / U2CELL as f32;
        let y = row as f32 / U2CELL as f32;

        // build済みのcell情報から指を取得
        let finger = self.cells[row][center_col].finger;
        let (hx, hy) = self.homes[&finger];

        let dx = (x - hx) as f64;
        let dy = (y - hy) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    /// Fitts time (calculated with given a, b parameters)
    pub fn fitts_time_from_start(&self, start: CellId, width_u: f32, a: f64, b: f64) -> f64 {
        let d = self.distance_u(start, width_u);
        fitts_time(a, b, d, width_u as f64)
    }
}

/// Euclidean distance in u units
pub fn euclid_u(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}
