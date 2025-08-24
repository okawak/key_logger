use crate::{
    constants::{COLUMN_STAGGER, MAX_ROW, MIN_ROW, ORTHO, ROW_STAGGER},
    error::{KbOptError, Result},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// メイン設定構造体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub solver: SolverConfig,
    pub v1: Option<V1Config>,
    pub v2: Option<V2Config>,
    pub v3: Option<V3Config>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SolverConfig {
    // 基本設定
    pub version: String, // "v1" | "v2" | "v3"
    pub output_dir: String,
    pub geometry: String, // "row-stagger" | "ortho" | "column-stagger"
    pub csv_dir: String,

    // 最適化設定
    #[serde(default)]
    pub include_fkeys: bool, // Fキーも最適化に含めるか(配置するかどうか)
    #[serde(default)]
    pub include_digits: bool, // 数字キーも最適化に含めるか(固定するかどうか)
    #[serde(default)]
    pub max_rows: usize, // geometryの行数（デフォルト6）
    #[serde(default)]
    pub align_left_edge: bool, // 左端揃え
    #[serde(default)]
    pub align_right_edge: bool, // 右端揃え
    #[serde(default)]
    pub solution_threshold: f64, // 解の閾値（デフォルト0.5）

    // Fitts係数
    pub fingerwise_coeffs: Option<HashMap<String, FittsCoefficient>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FittsCoefficient {
    pub a_ms: f64,
    pub b_ms: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct V1Config {
    /// 数字クラスタ設定
    #[serde(default)]
    pub digit_cluster: DigitClusterConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DigitClusterConfig {
    pub enforce_sequence: bool,
    pub enforce_horizontal: bool,
    pub allowed_rows: Vec<usize>,
}

impl Default for DigitClusterConfig {
    fn default() -> Self {
        Self {
            enforce_sequence: true,
            enforce_horizontal: true,
            allowed_rows: vec![4], // デフォルトでは通常の配列と同じ(下から)4行目のみ
        }
    }
}

// まだ未実装
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct V2Config {
    #[serde(default)]
    pub max_layers: usize,
}

impl Default for V2Config {
    fn default() -> Self {
        Self { max_layers: 1 }
    }
}

// まだ未実装
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct V3Config {
    #[serde(default)]
    pub max_layers: usize,
}

impl Default for V3Config {
    fn default() -> Self {
        Self { max_layers: 1 }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            solver: SolverConfig {
                version: String::new(),
                output_dir: String::new(),
                geometry: String::new(),
                csv_dir: String::new(),
                include_fkeys: false,
                include_digits: false,
                max_rows: 6,
                align_left_edge: false,
                align_right_edge: false,
                solution_threshold: 0.5,
                fingerwise_coeffs: None,
            },
            v1: None,
            v2: None,
            v3: None,
        }
    }
}

impl Config {
    /// 設定ファイルから読み込み
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            KbOptError::Config(format!(
                "Failed to read config file '{}': {}",
                path.display(),
                e
            ))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            KbOptError::Config(format!(
                "Failed to parse config file '{}': {}",
                path.display(),
                e
            ))
        })?;

        config.validate()?;
        Ok(config)
    }

    /// 設定の検証
    pub fn validate(&self) -> Result<()> {
        // max_rowsの検証
        if self.solver.max_rows < MIN_ROW || self.solver.max_rows > MAX_ROW {
            return Err(KbOptError::Config(format!(
                "max_rows must be between {} and {} (currently), got {}",
                MIN_ROW, MAX_ROW, self.solver.max_rows
            )));
        }

        // バージョンの検証
        match self.solver.version.as_str() {
            "v1" => self.validate_v1_config()?,
            "v2" => self.validate_v2_config()?,
            "v3" => self.validate_v3_config()?,
            _ => {
                return Err(KbOptError::Config(format!(
                    "Invalid solver version: {}. Must be 'v1', 'v2', or 'v3'",
                    self.solver.version
                )));
            }
        }

        // ジオメトリの検証
        match self.solver.geometry.as_str() {
            ROW_STAGGER | ORTHO | COLUMN_STAGGER => {}
            _ => {
                return Err(KbOptError::Config(format!(
                    "Invalid geometry: {}. Must be 'row-stagger', 'ortho', or 'column-stagger'",
                    self.solver.geometry
                )));
            }
        }

        Ok(())
    }

    fn validate_v1_config(&self) -> Result<()> {
        let solver_config = &self.solver;
        // 単一レイヤーなので、Fキーを入れる場合は、十分な行数が必要 (Fキー行 + 数字行)
        if solver_config.include_fkeys && solver_config.max_rows < MIN_ROW + 2 {
            return Err(KbOptError::Config(format!(
                "include_fkeys is true, but max_rows < {}",
                MIN_ROW + 2
            )));
        }

        if let Some(v1config) = &self.v1 {
            if solver_config.include_digits && v1config.digit_cluster.allowed_rows.is_empty() {
                return Err(KbOptError::Config(
                    "digit_cluster.allowed_rows cannot be empty when include_digits is true"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }

    // まだ未実装
    fn validate_v2_config(&self) -> Result<()> {
        Err(KbOptError::Config("v2 is under development".to_string()))
    }

    // まだ未実装
    fn validate_v3_config(&self) -> Result<()> {
        Err(KbOptError::Config("v3 is under development".to_string()))
    }

    pub fn debug_print(&self) {
        println!("{:#?}", self);
    }
}
