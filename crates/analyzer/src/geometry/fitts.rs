use super::types::*;

/// Fitts' law (v1: W_eff = w)
#[inline]
pub fn fitts_time(a: f64, b: f64, dist_u: f64, width_u: f64) -> f64 {
    a + b * ((dist_u / width_u) + 1.0).log2()
}

impl Geometry {
    /// Distance from finger home to key center [u]
    pub fn distance_u(&self, start: CellId, width_u: f32) -> f64 {
        let row = start.row;
        let center_col = start.col + cells_from_u(width_u) / 2;
        let x = self.cfg.rows[row].offset_u + (center_col as f32) * CELL_U;
        let mut y = self.cfg.rows[row].base_y_u;
        if matches!(self.name, GeometryName::ColStagger) {
            y += self.cfg.col_stagger_y[center_col];
        }
        let finger = if row == 0 {
            if x < 7.5 {
                Finger::LThumb
            } else {
                Finger::RThumb
            }
        } else {
            finger_from_x(x, &self.cfg.finger_x_boundaries)
        };
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
