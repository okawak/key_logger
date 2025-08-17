use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;
use crate::geometry::Geometry;
use std::path::PathBuf;
use std::time::Instant;

/// バージョン比較結果
#[derive(Debug, Clone)]
pub struct VersionComparison {
    pub v1_result: super::SolutionLayout,
    pub v2_result: super::SolutionLayout,
    pub v1_geometry: Geometry,
    pub v2_geometry: Geometry,
    pub comparison_metadata: ComparisonMetadata,
}

#[derive(Debug, Clone)]
pub struct ComparisonMetadata {
    pub timestamp: String,
    pub v1_solve_time_ms: u64,
    pub v2_solve_time_ms: u64,
    pub improvement_percent: f64,
    pub key_diff_count: usize,
}

impl VersionComparison {
    /// Create compare directory helper method
    fn create_compare_dir() -> Result<PathBuf, KbOptError> {
        let compare_dir = PathBuf::from("compare");
        std::fs::create_dir_all(&compare_dir).map_err(|e| {
            KbOptError::IoError(format!("Failed to create compare directory: {}", e))
        })?;
        Ok(compare_dir)
    }
    pub fn new(
        v1_result: super::SolutionLayout,
        v2_result: super::SolutionLayout,
        v1_geometry: Geometry,
        v2_geometry: Geometry,
        v1_solve_time_ms: u64,
        v2_solve_time_ms: u64,
    ) -> Self {
        let improvement_percent = if v1_result.objective_ms > 0.0 {
            100.0 * (v1_result.objective_ms - v2_result.objective_ms) / v1_result.objective_ms
        } else {
            0.0
        };

        let key_diff_count = count_key_placement_differences(&v1_geometry, &v2_geometry);

        Self {
            v1_result,
            v2_result,
            v1_geometry,
            v2_geometry,
            comparison_metadata: ComparisonMetadata {
                timestamp: chrono::Utc::now().to_rfc3339(),
                v1_solve_time_ms,
                v2_solve_time_ms,
                improvement_percent,
                key_diff_count,
            },
        }
    }

    pub fn save_report(&self, format: &str) -> Result<(), KbOptError> {
        match format {
            "html" => self.save_html_report(),
            "json" => self.save_json_report(),
            "csv" => self.save_csv_report(),
            _ => Err(KbOptError::ConfigError(format!(
                "Unknown report format: {}",
                format
            ))),
        }
    }

    fn save_html_report(&self) -> Result<(), KbOptError> {
        let html_content = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>v1 vs v2 Comparison Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .comparison-table {{ border-collapse: collapse; width: 100%; }}
        .comparison-table th, .comparison-table td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        .improvement {{ color: green; font-weight: bold; }}
        .degradation {{ color: red; font-weight: bold; }}
        .neutral {{ color: gray; }}
    </style>
</head>
<body>
    <h1>キーボード配列最適化 v1 vs v2 比較レポート</h1>
    <p>生成日時: {}</p>
    
    <h2>最適化結果比較</h2>
    <table class="comparison-table">
        <tr><th>項目</th><th>v1</th><th>v2</th><th>差分</th></tr>
        <tr><td>目的関数値 (ms)</td><td>{:.3}</td><td>{:.3}</td><td class="{}">{:.3}</td></tr>
        <tr><td>求解時間 (ms)</td><td>{}</td><td>{}</td><td class="neutral">{}</td></tr>
        <tr><td>配置変更キー数</td><td colspan="2">{}</td><td class="neutral">-</td></tr>
    </table>
    
    <h2>改善率</h2>
    <p class="{}">v2による改善: {:.2}%</p>
    
    <h2>詳細情報</h2>
    <p>v1 実行時間: {} ms</p>
    <p>v2 実行時間: {} ms</p>
    <p>配置が変更されたキー数: {}</p>
    
</body>
</html>
        "#,
            self.comparison_metadata.timestamp,
            self.v1_result.objective_ms,
            self.v2_result.objective_ms,
            if self.comparison_metadata.improvement_percent > 0.0 {
                "improvement"
            } else {
                "degradation"
            },
            self.v1_result.objective_ms - self.v2_result.objective_ms,
            self.comparison_metadata.v1_solve_time_ms,
            self.comparison_metadata.v2_solve_time_ms,
            (self.comparison_metadata.v2_solve_time_ms as i64)
                - (self.comparison_metadata.v1_solve_time_ms as i64),
            self.comparison_metadata.key_diff_count,
            if self.comparison_metadata.improvement_percent > 0.0 {
                "improvement"
            } else {
                "degradation"
            },
            self.comparison_metadata.improvement_percent,
            self.comparison_metadata.v1_solve_time_ms,
            self.comparison_metadata.v2_solve_time_ms,
            self.comparison_metadata.key_diff_count,
        );

        let compare_dir = Self::create_compare_dir()?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = compare_dir.join(format!("comparison_report_{}.html", timestamp));
        std::fs::write(&filename, html_content)
            .map_err(|e| KbOptError::IoError(format!("Failed to save HTML report: {}", e)))?;
        log::info!("Comparison report saved to: {}", filename.display());
        Ok(())
    }

    fn save_json_report(&self) -> Result<(), KbOptError> {
        let json_data = serde_json::json!({
            "timestamp": self.comparison_metadata.timestamp,
            "v1": {
                "objective_ms": self.v1_result.objective_ms,
                "solve_time_ms": self.comparison_metadata.v1_solve_time_ms,
            },
            "v2": {
                "objective_ms": self.v2_result.objective_ms,
                "solve_time_ms": self.comparison_metadata.v2_solve_time_ms,
            },
            "comparison": {
                "improvement_percent": self.comparison_metadata.improvement_percent,
                "key_diff_count": self.comparison_metadata.key_diff_count,
                "objective_diff_ms": self.v1_result.objective_ms - self.v2_result.objective_ms,
            }
        });

        let compare_dir = Self::create_compare_dir()?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = compare_dir.join(format!("comparison_report_{}.json", timestamp));
        std::fs::write(&filename, serde_json::to_string_pretty(&json_data)?)
            .map_err(|e| KbOptError::IoError(format!("Failed to save JSON report: {}", e)))?;
        log::info!("Comparison report saved to: {}", filename.display());
        Ok(())
    }

    fn save_csv_report(&self) -> Result<(), KbOptError> {
        let csv_content = format!(
            "timestamp,v1_objective_ms,v2_objective_ms,improvement_percent,v1_solve_time_ms,v2_solve_time_ms,key_diff_count\n{},{},{},{},{},{},{}\n",
            self.comparison_metadata.timestamp,
            self.v1_result.objective_ms,
            self.v2_result.objective_ms,
            self.comparison_metadata.improvement_percent,
            self.comparison_metadata.v1_solve_time_ms,
            self.comparison_metadata.v2_solve_time_ms,
            self.comparison_metadata.key_diff_count,
        );

        let compare_dir = Self::create_compare_dir()?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = compare_dir.join(format!("comparison_report_{}.csv", timestamp));
        std::fs::write(&filename, csv_content)
            .map_err(|e| KbOptError::IoError(format!("Failed to save CSV report: {}", e)))?;
        log::info!("Comparison report saved to: {}", filename.display());
        Ok(())
    }
}

/// キー配置の差分をカウント
fn count_key_placement_differences(geom_v1: &Geometry, geom_v2: &Geometry) -> usize {
    let keys_v1: std::collections::HashSet<_> = geom_v1.key_placements.keys().collect();
    let keys_v2: std::collections::HashSet<_> = geom_v2.key_placements.keys().collect();

    let mut diff_count = 0;
    for key in keys_v1.union(&keys_v2) {
        match (
            geom_v1.key_placements.get(*key),
            geom_v2.key_placements.get(*key),
        ) {
            (Some(p1), Some(p2)) => {
                // 位置や幅に違いがあるかチェック
                if (p1.x - p2.x).abs() > 1e-6
                    || (p1.y - p2.y).abs() > 1e-6
                    || (p1.width_u - p2.width_u).abs() > 1e-6
                {
                    diff_count += 1;
                }
            }
            _ => diff_count += 1, // どちらかにのみ存在する場合
        }
    }

    diff_count
}

/// 時間測定ヘルパー
pub struct TimedExecution<T> {
    pub result: T,
    pub duration_ms: u64,
}

impl<T> TimedExecution<T> {
    pub fn time<F>(f: F) -> TimedExecution<T>
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = f();
        let duration_ms = start.elapsed().as_millis() as u64;

        TimedExecution {
            result,
            duration_ms,
        }
    }
}

/// v1とv2の比較実行
pub fn execute_comparison(
    geom: &Geometry,
    freqs: &KeyFreq,
    v1_opts: &super::SolveOptions,
) -> Result<VersionComparison, KbOptError> {
    // v1実行
    let mut geom_v1 = geom.clone();
    let v1_execution =
        TimedExecution::time(|| super::v1::solve_layout_v1(&mut geom_v1, freqs, v1_opts));
    let v1_result = v1_execution.result?;

    // v2実行（Phase 1設定で指別係数を使用）
    let mut geom_v2 = geom.clone();
    let v2_opts = super::SolveOptionsV2 {
        base: v1_opts.clone(),
        fitts_coeffs: Some(super::config::FittsCoefficientsConfig {
            enable: true,
            values: Some(std::collections::HashMap::from([
                ("LThumb".to_string(), [50.0, 140.0]),
                ("LIndex".to_string(), [40.0, 120.0]),
                ("LMiddle".to_string(), [45.0, 130.0]),
                ("LRing".to_string(), [55.0, 145.0]),
                ("LPinky".to_string(), [65.0, 160.0]),
                ("RThumb".to_string(), [50.0, 140.0]),
                ("RIndex".to_string(), [40.0, 120.0]),
                ("RMiddle".to_string(), [45.0, 130.0]),
                ("RRing".to_string(), [55.0, 145.0]),
                ("RPinky".to_string(), [65.0, 160.0]),
            ])),
        }),
        directional_width: None,
        layers: None,
        digits: None,
        bigrams: None,
    };

    let v2_execution =
        TimedExecution::time(|| super::v2::solve_layout_v2(&mut geom_v2, freqs, &v2_opts));
    let v2_result = v2_execution.result?;

    Ok(VersionComparison::new(
        v1_result,
        v2_result,
        geom_v1,
        geom_v2,
        v1_execution.duration_ms,
        v2_execution.duration_ms,
    ))
}
