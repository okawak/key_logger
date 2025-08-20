/// # レイヤシステム実装 (v2準備版)
///
/// CLAUDE.md v2仕様に基づく複数レイヤー・アルファベット固定モデル用の実装
///
/// ## 概要
/// - アルファベットキー（A-Z）は固定位置（QWERTY等）
/// - ベースレイヤー + 複数レイヤー（モディファイア併用）
/// - 同時押し時間の最適化（並行係数対応）
///
/// ## 主要機能
/// 1. **LayerConfig**: レイヤシステムの設定管理
/// 2. **LayerVariables**: レイヤ関連の決定変数群
/// 3. **compute_chord_time**: 同時押し時間計算（並行係数対応）
/// 4. **generate_layer_constraints**: レイヤ制約の生成
///
/// ## 数理モデル対応
/// - q_{l,m}: レイヤlのモディファイアをブロックmに配置
/// - z_{s,l,u,m}: 記号sをレイヤlのブロックuに配置（モディファイアm使用）
/// - 同時押し時間: T^chord = T_main + T_modifier - θ·min{T_main, T_modifier} + τ_mod
use crate::geometry::Geometry;
use crate::geometry::types::Finger;
use crate::optimize::config::LayersConfig;
use crate::optimize::fitts::FingerwiseFittsCoefficients;
use good_lp::Variable;
use std::collections::HashMap;

/// v2レイヤシステムの設定構造体（実行時用）
///
/// CLAUDE.md v2仕様の実装:
/// - モディファイアアンカー: $\tilde{M} \subset \mathcal{U}_g$ (親指行等の1uブロック)  
/// - 並行係数: $\theta_{f,f'}$ (異なる指の組み合わせでの同時押し効率)
/// - モディファイアペナルティ: $\tau_{\mathrm{mod}}$ (レイヤ切替コスト)
#[derive(Debug, Clone)]
pub struct LayerConfig {
    /// レイヤ機能の有効化フラグ
    pub enable_layers: bool,
    /// モディファイア配置可能なブロックID集合 $\tilde{M}$
    pub modifier_anchors: Vec<usize>,
    /// 並行係数 $\theta_{f,f'}$ マップ（指の組み合わせ → 並行効率）
    pub parallel_coefficients: HashMap<(Finger, Finger), f64>,
    /// モディファイアペナルティ $\tau_{\mathrm{mod}}$ (ms)
    pub modifier_penalty_ms: f64,
    /// 最大レイヤ数（v2では通常1-3）
    pub max_layers: usize,
}

impl LayerConfig {
    /// LayersConfigから変換
    pub fn from_config(config: &LayersConfig) -> Self {
        // modifier_rowsから実際のブロックIDを導出する簡易実装
        // TODO: より詳細な設定が必要な場合は拡張
        let modifier_anchors: Vec<usize> = config
            .modifier_rows
            .iter()
            .flat_map(|&row| {
                // 各行の左右端のブロックを想定（親指キー配置想定）
                if row >= 3 {
                    // 親指行想定
                    vec![row * 10, row * 10 + 1] // 簡易的なブロックID計算
                } else {
                    vec![]
                }
            })
            .collect();

        Self {
            enable_layers: config.enable,
            modifier_anchors,
            parallel_coefficients: Self::default_parallel_coefficients(),
            modifier_penalty_ms: config.modifier_penalty_ms,
            max_layers: config.max_layers.unwrap_or(1), // デフォルトは1レイヤー
        }
    }

    /// デフォルトの並行係数を生成
    fn default_parallel_coefficients() -> HashMap<(Finger, Finger), f64> {
        HashMap::from([
            // 異なる手の組み合わせ（高い並行性）
            ((Finger::LThumb, Finger::RIndex), 0.9),
            ((Finger::LThumb, Finger::RMiddle), 0.9),
            ((Finger::LThumb, Finger::RRing), 0.9),
            ((Finger::LThumb, Finger::RPinky), 0.9),
            ((Finger::LIndex, Finger::RThumb), 0.9),
            ((Finger::LMiddle, Finger::RThumb), 0.9),
            ((Finger::LRing, Finger::RThumb), 0.9),
            ((Finger::LPinky, Finger::RThumb), 0.9),
            // 同じ手の組み合わせ（低い並行性）
            ((Finger::LIndex, Finger::LMiddle), 0.4),
            ((Finger::LMiddle, Finger::LRing), 0.4),
            ((Finger::LRing, Finger::LPinky), 0.4),
            ((Finger::RIndex, Finger::RMiddle), 0.4),
            ((Finger::RMiddle, Finger::RRing), 0.4),
            ((Finger::RRing, Finger::RPinky), 0.4),
            // 隣接していない指の組み合わせ（中程度の並行性）
            ((Finger::LIndex, Finger::LRing), 0.6),
            ((Finger::LIndex, Finger::LPinky), 0.6),
            ((Finger::RIndex, Finger::RRing), 0.6),
            ((Finger::RIndex, Finger::RPinky), 0.6),
        ])
    }
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            enable_layers: false,
            modifier_anchors: vec![],
            parallel_coefficients: Self::default_parallel_coefficients(),
            modifier_penalty_ms: 10.0,
            max_layers: 3,
        }
    }
}

/// レイヤシステムの決定変数群
#[derive(Debug)]
pub struct LayerVariables {
    /// q_{l,m}: レイヤlのモディファイアをブロックmに配置
    pub q_vars: HashMap<(usize, usize), Variable>,
    /// z_{s,l,u,m}: 記号sをレイヤlのブロックuに配置（モディファイアm使用）
    pub z_vars: HashMap<(crate::keys::KeyId, usize, usize, usize), Variable>,
    /// x^{mod}_{k,u}: モディファイアキーkをブロックuに配置
    pub modifier_placement_vars: HashMap<(crate::keys::KeyId, usize), Variable>,
}

/// v2同時押し時間の計算
///
/// CLAUDE.md v2仕様の実装:
/// ```
/// T^{chord}(u,m) = T_tap(u) + T_tap(m) - θ_{F(u),F(m)} min{T_tap(u), T_tap(m)} + τ_mod
/// ```
///
/// 並行係数θにより、同じ手・異なる手での同時押し効率を考慮
/// - 異なる手: θ ≈ 0.9 (高い並行性)
/// - 同じ手: θ ≈ 0.4 (低い並行性)
///
/// # Arguments
/// * `main_block_center` - メインキーのブロック中心座標 (mm)
/// * `main_finger` - メインキーを担当する指
/// * `modifier_block_center` - モディファイアキーのブロック中心座標 (mm)  
/// * `modifier_finger` - モディファイアキーを担当する指
/// * `geom` - ジオメトリ情報
/// * `fitts_coeffs` - 指別Fitts係数
/// * `layer_cfg` - レイヤ設定
///
/// # Returns
/// 同時押し時間 (ms)
pub fn compute_chord_time(
    main_block_center: (f32, f32),
    main_finger: Finger,
    modifier_block_center: (f32, f32),
    modifier_finger: Finger,
    geom: &Geometry,
    fitts_coeffs: &FingerwiseFittsCoefficients,
    layer_cfg: &LayerConfig,
) -> f64 {
    use crate::optimize::fitts::compute_unified_fitts_time;

    // メインキーのホームポジション
    let main_home = geom
        .homes
        .get(&main_finger)
        .cloned()
        .unwrap_or(main_block_center);
    let _main_distance = crate::constants::euclid_distance(main_block_center, main_home) as f64;
    // TODO: 方向角計算の実装
    // let main_direction = compute_direction_angle(main_home, main_block_center);

    // モディファイアキーのホームポジション
    let modifier_home = geom
        .homes
        .get(&modifier_finger)
        .cloned()
        .unwrap_or(modifier_block_center);
    let _modifier_distance =
        crate::constants::euclid_distance(modifier_block_center, modifier_home) as f64;
    // let modifier_direction = compute_direction_angle(modifier_home, modifier_block_center);

    // 各キーの単打時間を計算（共通Fitts機能を使用）
    let t_main = compute_unified_fitts_time(
        main_finger,
        main_block_center,
        main_home,
        1.0,  // ブロックは1u想定
        true, // 指別係数を使用
        fitts_coeffs,
        50.0,  // デフォルトa_ms
        150.0, // デフォルトb_ms
    );

    let t_modifier = compute_unified_fitts_time(
        modifier_finger,
        modifier_block_center,
        modifier_home,
        1.0,  // ブロックは1u想定
        true, // 指別係数を使用
        fitts_coeffs,
        50.0,  // デフォルトa_ms
        150.0, // デフォルトb_ms
    );

    // 並行係数θを取得
    let theta = layer_cfg
        .parallel_coefficients
        .get(&(main_finger, modifier_finger))
        .or_else(|| {
            layer_cfg
                .parallel_coefficients
                .get(&(modifier_finger, main_finger))
        })
        .copied()
        .unwrap_or(0.0); // デフォルトは並行性なし

    // 同時押し時間の計算（a値重複問題を修正）
    // オリジナル: T^chord = T_main + T_modifier - θ·min(T_main, T_modifier) + τ_mod
    // 修正版: a値の重複を避けるため、max(T_main, T_modifier)ベースに変更
    // T^chord = max(T_main, T_modifier) + (1-θ)·min(T_main, T_modifier) + τ_mod
    let t_max = t_main.max(t_modifier);
    let t_min = t_main.min(t_modifier);
    t_max + (1.0 - theta) * t_min + layer_cfg.modifier_penalty_ms
}

/// レイヤ制約の生成
///
/// 以下の制約を追加：
/// 1. モディファイア一意性制約: sum_m q_{l,m} <= 1
/// 2. レイヤ配置整合性制約: z_{s,l,u,m} <= q_{l,m}  
/// 3. 記号配置一意性制約: sum_j x_{s,j,w} + sum_{l,u,m} z_{s,l,u,m} = 1
///
/// レイヤ制約を制約リストとして生成（solver.rsで使用）
pub fn generate_layer_constraints(
    layer_vars: &LayerVariables,
    layer_cfg: &LayerConfig,
    modifier_keys: &[crate::keys::KeyId],
    layer_candidate_keys: &[crate::keys::KeyId],
) -> Vec<good_lp::Constraint> {
    use good_lp::Expression;
    let mut constraints = Vec::new();

    // 1. モディファイアキー配置制約: 各モディファイアキーは必ず1つの場所に配置
    for &modifier_key in modifier_keys {
        let mut sum = Expression::from(0.0);
        for (key, _u) in layer_vars.modifier_placement_vars.keys() {
            if *key == modifier_key {
                sum += layer_vars.modifier_placement_vars[&(*key, *_u)];
            }
        }
        constraints.push(sum.eq(1.0)); // 必ず配置するように変更
    }

    // 2. モディファイア一意性制約: 各レイヤには最大1つのモディファイア
    for l in 1..=layer_cfg.max_layers {
        let mut sum = Expression::from(0.0);
        for &m in &layer_cfg.modifier_anchors {
            if let Some(&q_var) = layer_vars.q_vars.get(&(l, m)) {
                sum += q_var;
            }
        }
        constraints.push(sum << 1.0);
    }

    // 3. レイヤ配置整合性制約: レイヤ記号はモディファイアが配置されているときのみ
    for ((_, l, _u, m), &z_var) in &layer_vars.z_vars {
        if *l >= 1 {
            // ベースレイヤ以外
            if let Some(&q_var) = layer_vars.q_vars.get(&(*l, *m)) {
                constraints.push(z_var << q_var);
            }
        }
    }

    // 3.5. モディファイア配置と使用の整合性制約: q_{l,m} <= sum_k x^{mod}_{k,m}
    for l in 1..=layer_cfg.max_layers {
        for &m in &layer_cfg.modifier_anchors {
            if let Some(&q_var) = layer_vars.q_vars.get(&(l, m)) {
                let mut modifier_sum = Expression::from(0.0);
                for &modifier_key in modifier_keys {
                    if let Some(&mod_var) =
                        layer_vars.modifier_placement_vars.get(&(modifier_key, m))
                    {
                        modifier_sum += mod_var;
                    }
                }
                // q_{l,m} <= モディファイアキーがブロックmに配置されている
                constraints.push(q_var << modifier_sum);
            }
        }
    }

    // 3.6. レイヤー使用時のモディファイア強制配置: 簡素化版
    // レイヤ1が使用される場合、最低1つのモディファイアが必要
    if layer_cfg.max_layers >= 1 {
        let mut layer1_usage = Expression::from(0.0);
        for ((_, layer_num, _u, _m), &z_var) in &layer_vars.z_vars {
            if *layer_num == 1 {
                layer1_usage += z_var;
            }
        }

        let mut modifier_total = Expression::from(0.0);
        for &m in &layer_cfg.modifier_anchors {
            if let Some(&q_var) = layer_vars.q_vars.get(&(1, m)) {
                modifier_total += q_var;
            }
        }

        // レイヤ1が使用される場合はモディファイアが必要（簡素化）
        constraints.push(layer1_usage << (10.0 * modifier_total));
    }

    // 4. レイヤー間物理配置統一制約: 同一キーは全レイヤで同一物理位置に配置
    // 数理モデル「物理配置はレイヤ共通（レイヤで変化しない）」に対応
    for &key in layer_candidate_keys {
        for l1 in 1..=layer_cfg.max_layers {
            for l2 in (l1 + 1)..=layer_cfg.max_layers {
                // 同一キーkがレイヤl1とl2で異なる位置に配置されることを禁止
                for u1 in 0..layer_vars
                    .z_vars
                    .keys()
                    .map(|(_, _, u, _)| *u)
                    .max()
                    .unwrap_or(0)
                    + 1
                {
                    for u2 in 0..layer_vars
                        .z_vars
                        .keys()
                        .map(|(_, _, u, _)| *u)
                        .max()
                        .unwrap_or(0)
                        + 1
                    {
                        if u1 != u2 {
                            // キーkがレイヤl1のブロックu1とレイヤl2のブロックu2に同時に配置されることを禁止
                            let mut sum = Expression::from(0.0);

                            // レイヤl1のブロックu1の変数を追加
                            for &m1 in &layer_cfg.modifier_anchors {
                                if let Some(&z_var1) = layer_vars.z_vars.get(&(key, l1, u1, m1)) {
                                    sum += z_var1;
                                }
                            }

                            // レイヤl2のブロックu2の変数を追加
                            for &m2 in &layer_cfg.modifier_anchors {
                                if let Some(&z_var2) = layer_vars.z_vars.get(&(key, l2, u2, m2)) {
                                    sum += z_var2;
                                }
                            }

                            // 合計が1以下（同時に配置されない）
                            constraints.push(sum << 1.0);
                        }
                    }
                }
            }
        }
    }

    constraints
}

/// Phase 3: レイヤ決定変数の作成
pub fn create_layer_variables(
    vars: &mut good_lp::ProblemVariables,
    layer_candidate_keys: &[crate::keys::KeyId],
    num_blocks: usize,
    layer_cfg: &LayerConfig,
    modifier_keys: &[crate::keys::KeyId],
) -> LayerVariables {
    use good_lp::variable;

    let mut q_vars = HashMap::new();
    let mut z_vars = HashMap::new();
    let mut modifier_placement_vars = HashMap::new();

    // x^{mod}_{k,u}: モディファイアキーkをブロックuに配置（数理モデルに従い最適化対象）
    for &modifier_key in modifier_keys {
        for u in 0..num_blocks {
            modifier_placement_vars.insert((modifier_key, u), vars.add(variable().binary()));
        }
    }

    // q_{l,m}: レイヤlのモディファイアをブロックmに配置
    for l in 1..=layer_cfg.max_layers {
        for &m in &layer_cfg.modifier_anchors {
            q_vars.insert((l, m), vars.add(variable().binary()));
        }
    }

    // z_{k,l,u,m}: キーkをレイヤlのブロックuに配置（モディファイアm使用）
    for &key in layer_candidate_keys {
        for l in 1..=layer_cfg.max_layers {
            for u in 0..num_blocks {
                for &m in &layer_cfg.modifier_anchors {
                    z_vars.insert((key, l, u, m), vars.add(variable().binary()));
                }
            }
        }
    }

    LayerVariables {
        q_vars,
        z_vars,
        modifier_placement_vars,
    }
}

/// レイヤー候補キーを取得（頻度の高いキーを優先選択）
pub fn get_layer_candidate_keys_with_frequencies(
    parse_opt: &crate::keys::ParseOptions,
    probabilities: &std::collections::HashMap<crate::keys::KeyId, f64>,
) -> Vec<crate::keys::KeyId> {
    use crate::keys::all_movable_keys;

    let mut candidates = all_movable_keys(parse_opt)
        .into_iter()
        .filter(|key| {
            // アルファベットキーは除外（アルファベットキーはparse対象外なので、ここには含まれない）
            // モディファイアキーのみ除外（矢印キーはレイヤー候補に含める）
            !crate::optimize::v1::solver::is_modifier(key)
        })
        .collect::<Vec<_>>();

    // 頻度順にソート（降順）
    candidates.sort_by(|a, b| {
        let freq_a = probabilities.get(a).copied().unwrap_or(0.0);
        let freq_b = probabilities.get(b).copied().unwrap_or(0.0);
        freq_b
            .partial_cmp(&freq_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // テスト用に頻度の高い上位10個を選択
    candidates.into_iter().take(10).collect()
}

/// レイヤー候補キーを取得（全ての候補）
pub fn get_layer_candidate_keys(parse_opt: &crate::keys::ParseOptions) -> Vec<crate::keys::KeyId> {
    use crate::keys::all_movable_keys;

    all_movable_keys(parse_opt)
        .into_iter()
        .filter(|key| {
            // 矢印キーもレイヤー候補に含める
            // モディファイアキーのみ除外
            !crate::optimize::v1::solver::is_modifier(key)
        })
        .collect()
}

/// モディファイアキーの頻度計算
/// 数理モデルに従い、モディファイアキーの頻度 = そのレイヤーの全記号頻度の合計
pub fn compute_modifier_frequency(
    _layer_num: usize,
    layer_candidate_keys: &[crate::keys::KeyId],
    key_frequencies: &std::collections::HashMap<crate::keys::KeyId, f64>,
) -> f64 {
    layer_candidate_keys
        .iter()
        .map(|key| key_frequencies.get(key).copied().unwrap_or(0.0))
        .sum()
}
