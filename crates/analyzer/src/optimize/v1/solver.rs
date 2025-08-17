// v1ソルバー実装: 単一係数Fitts法則を使用した最適化

use std::collections::{BTreeSet, HashMap};

use super::fitts::compute_key_fitts_time;
use crate::constants::U2CELL;
use crate::geometry::{
    Geometry,
    sets::{OptimizationSets, extract_free_cell_intervals},
    types::{BlockId, CellId, KeyCandidates},
};
use crate::keys::{ArrowKey, KeyId};

/// 矢印キー定数
pub const ARROW_KEYS: [KeyId; 4] = [
    KeyId::Arrow(ArrowKey::Up),
    KeyId::Arrow(ArrowKey::Down),
    KeyId::Arrow(ArrowKey::Left),
    KeyId::Arrow(ArrowKey::Right),
];

/// キー種別判定
pub fn is_arrow(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Arrow(_))
}

pub fn is_digit_or_f(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Digit(_) | KeyId::Function(_))
}

/// 幅候補（0.25u 刻み）。数字/F/矢印は 1u 固定。
pub fn width_candidates_for_key(key_id: &KeyId) -> Vec<f32> {
    if is_arrow(key_id) || is_digit_or_f(key_id) {
        vec![1.0]
    } else {
        // 0.25u 刻みで 1.00..2.00 あたり（最小幅1uを保証）
        let mut v = Vec::new();
        let mut w = 1.00f32;
        while w <= 2.00 + 1e-6 {
            v.push((w * 100.0).round() / 100.0);
            w += 0.25;
        }
        v
    }
}

/// 配置候補（通常キー）
#[derive(Debug, Clone)]
pub struct Cand {
    pub key: KeyId,
    pub row: usize,
    pub start_col: usize, // 0.25u index
    pub w_u: f32,
    pub cost_ms: f64, // f_k を掛ける前の素コスト
    pub cover_cells: Vec<CellId>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: BlockId,
    pub center: (f32, f32),       // [u]
    pub cover_cells: [CellId; 4], // この1uが覆う 0.25u セル
}

/// v1実装のメインエントリポイント
pub fn solve_layout_v1(
    geom: &mut Geometry,
    freqs: &crate::csv_reader::KeyFreq,
    opt: &crate::optimize::SolveOptions,
) -> Result<crate::optimize::SolutionLayout, crate::error::KbOptError> {
    use crate::geometry::types::{CellId, PlacementType};
    use crate::keys::{KeyId, ParseOptions, all_movable_keys};
    use good_lp::{
        Expression, ProblemVariables, Solution, SolverModel, Variable, coin_cbc, variable,
    };
    use std::collections::{BTreeSet, HashMap};

    /// 最適化モデルの定数
    const REQUIRED_ARROW_BLOCKS: usize = 4;
    const REQUIRED_ARROW_BLOCKS_F64: f64 = 4.0;
    const FLOW_ROOTS_F64: f64 = 1.0;
    const MAX_FLOW_PER_BLOCK: f64 = 3.0;

    // 1) 集合を作る - CSVにないキーも含める（count 0として扱う）
    let parse_opt = ParseOptions {
        include_fkeys: opt.include_fkeys,
        ..Default::default()
    };

    let movable: BTreeSet<KeyId> = all_movable_keys(&parse_opt)
        .into_iter()
        .filter(|k| !is_arrow(k))
        .collect();

    // 2) v1: 全キーが全行に配置可能、全空きセルが矢印キー候補
    let movable_vec: Vec<KeyId> = movable.iter().cloned().collect();
    let key_cands = generate_v1_key_candidates(geom, &movable_vec);
    let (arrow_cells, arrow_edges) = generate_v1_arrow_region(geom);

    let optimization_sets = OptimizationSets {
        key_cands,
        arrow_cells,
        arrow_edges,
    };

    // 3) OptimizationSetsから通常キーの候補を変換
    let cands = build_candidates_from_precompute(geom, &movable, &optimization_sets, opt);
    if cands.is_empty() {
        return Err(crate::error::KbOptError::ModelError {
            message: "No valid key placement candidates found".to_string(),
        });
    }

    // 4) OptimizationSetsから矢印用ブロックを変換
    let (blocks, _block_index) = build_blocks_from_precompute(geom, &optimization_sets);
    if blocks.len() < REQUIRED_ARROW_BLOCKS {
        return Err(crate::error::KbOptError::ModelError {
            message: format!(
                "Insufficient arrow blocks: found {}, need at least {}",
                blocks.len(),
                REQUIRED_ARROW_BLOCKS
            ),
        });
    }

    let adj_edges = build_adjacency_from_precompute(&blocks, &optimization_sets);

    // 4) モデルを立てる
    let mut vars = ProblemVariables::new();

    // x^g_{k,j,w}（二値）：通常キー配置変数
    let x_vars: Vec<Variable> = (0..cands.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // a^g_u（二値）：ブロック占有変数
    let a_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // m^g_{a,u}：矢印キー配置変数（4種×ブロック）
    let mut m_vars: HashMap<(KeyId, usize), Variable> = HashMap::new();
    for &arrow_key in &ARROW_KEYS {
        for u in 0..blocks.len() {
            m_vars.insert((arrow_key, u), vars.add(variable().binary()));
        }
    }

    // r^g_u（二値）：フロー根変数
    let r_vars: Vec<Variable> = (0..blocks.len())
        .map(|_| vars.add(variable().binary()))
        .collect();

    // f^g_{(u→v)}（連続）：フロー変数
    #[derive(Clone)]
    struct Edge {
        from: usize,
        to: usize,
    }
    let edges: Vec<Edge> = adj_edges
        .iter()
        .map(|(u, v)| Edge { from: *u, to: *v })
        .collect();
    let f_vars: Vec<Variable> = (0..edges.len())
        .map(|_| vars.add(variable().min(0.0)))
        .collect();

    // 正規化された確率値を取得
    let probabilities = freqs.probabilities();

    // 目的関数: Σ p_k·T^g(j,w)·x^g_{k,j,w} + Σ p_a·T^g_arrow(u)·m^g_{a,u}
    let mut obj = Expression::from(0.0);
    // 通常キー項: Σ_{k∈K} Σ_{j∈I^g_k} Σ_{w∈W_k} p_k·T^g(j,w)·x^g_{k,j,w}
    for (i, cand) in cands.iter().enumerate() {
        let p_k = probabilities.get(&cand.key).copied().unwrap_or(0.0);
        // 頻度が0でも小さな値（1e-6）を設定して目的関数に含める
        let effective_p_k = if p_k > 0.0 { p_k } else { 1e-6 };
        obj += effective_p_k * cand.cost_ms * x_vars[i];
    }
    // 矢印キー項: Σ_{a∈A} Σ_{u∈U^g} p_a·T^g_arrow(u)·m^g_{a,u}
    for (u, blk) in blocks.iter().enumerate() {
        let center_cell = blk.cover_cells[2]; // 中央近傍
        let finger = geom.cells[center_cell.row][center_cell.col].finger;
        let home = geom.homes.get(&finger).cloned().unwrap_or((
            blk.center.0 * crate::constants::U2MM as f32,
            blk.center.1 * crate::constants::U2MM as f32,
        ));

        // v1 Fitts計算を使用（blk.centerをmm単位に変換）
        let center_mm = (
            blk.center.0 * crate::constants::U2MM as f32,
            blk.center.1 * crate::constants::U2MM as f32,
        );
        let t_ms = compute_key_fitts_time(center_mm, home, 1.0, opt.a_ms, opt.b_ms);

        for &arrow_key in &ARROW_KEYS {
            let p_a = probabilities.get(&arrow_key).copied().unwrap_or(0.0);
            // 頻度が0でも小さな値（1e-6）を設定して目的関数に含める
            let effective_p_a = if p_a > 0.0 { p_a } else { 1e-6 };
            let m_au = m_vars.get(&(arrow_key, u)).unwrap();
            obj += (effective_p_a * t_ms) * *m_au;
        }
    }

    // 目的関数を後で評価するために保存
    let objective_expr = obj.clone();

    // 5) 制約条件

    let mut model = vars.minimise(obj).using(coin_cbc);

    // (i) 一意性制約: Σ_{j∈I^g_k} Σ_{w∈W_k} x^g_{k,j,w} = 1 ∀k∈K
    // movable集合に含まれている全キーは必須配置（頻度0でも配置する）
    for &key in movable.iter() {
        let idxs: Vec<usize> = cands
            .iter()
            .enumerate()
            .filter(|(_, c)| c.key == key)
            .map(|(i, _)| i)
            .collect();
        if !idxs.is_empty() {
            let sum: Expression = idxs.iter().map(|i| x_vars[*i]).sum();
            // movable集合のキーは全て必須配置
            model = model.with(sum.clone().eq(1));
        }
    }

    // (ii) セル非重複制約: Σ C(j',j,w)·x^g_{k,j,w} + Σ B(j',u)·a^g_u + F^g_{j'} ≤ 1 ∀j'∈J_g
    let mut cell_cover_x: HashMap<CellId, Vec<usize>> = HashMap::new();
    for (i, cand) in cands.iter().enumerate() {
        for cid in &cand.cover_cells {
            cell_cover_x.entry(*cid).or_default().push(i);
        }
    }
    let mut cell_cover_a: HashMap<CellId, Vec<usize>> = HashMap::new();
    for (u, blk) in blocks.iter().enumerate() {
        for cid in &blk.cover_cells {
            cell_cover_a.entry(*cid).or_default().push(u);
        }
    }
    for r in 0..geom.cells.len() {
        for c in 0..geom.cells[r].len() {
            let cid = CellId::new(r, c);
            let fixed = if geom.cells[r][c].occupied { 1.0 } else { 0.0 };
            let xs = cell_cover_x.get(&cid).cloned().unwrap_or_default();
            let as_ = cell_cover_a.get(&cid).cloned().unwrap_or_default();
            let mut sum = Expression::from(fixed);
            for i in xs {
                sum += x_vars[i];
            }
            for u in as_ {
                sum += a_vars[u];
            }
            model = model.with(sum << 1);
        }
    }

    // (iii) 矢印キー一意性制約: Σ_{u∈U_g} m^g_{a,u} = 1 ∀a∈A
    for &arrow_key in &ARROW_KEYS {
        let sum: Expression = (0..blocks.len())
            .map(|u| *m_vars.get(&(arrow_key, u)).unwrap())
            .sum();
        model = model.with(sum.eq(1));
    }
    // (iv) ブロック占有整合性制約: Σ_{a∈A} m^g_{a,u} ≤ a^g_u ∀u∈U^g
    for (u, _) in a_vars.iter().enumerate().take(blocks.len()) {
        let sum_a: Expression = ARROW_KEYS
            .iter()
            .map(|&arrow_key| *m_vars.get(&(arrow_key, u)).unwrap())
            .sum();
        model = model.with(sum_a << a_vars[u]);
    }
    // (v) 矢印キー総数制約: Σ_{u∈U_g} a^g_u = REQUIRED_ARROW_BLOCKS
    let sum_a_total: Expression = (0..blocks.len()).map(|u| a_vars[u]).sum();
    model = model.with(sum_a_total.eq(REQUIRED_ARROW_BLOCKS_F64));

    // (vi) 矢印キー連結制約（フロー保存）
    // フロー根一意性: Σ_{u∈U_g} r^g_u = FLOW_ROOTS
    let sum_r: Expression = (0..blocks.len()).map(|u| r_vars[u]).sum();
    model = model.with(sum_r.eq(FLOW_ROOTS_F64));

    // 出入辺リスト
    let mut in_edges: Vec<Vec<usize>> = vec![Vec::new(); blocks.len()];
    let mut out_edges: Vec<Vec<usize>> = vec![Vec::new(); blocks.len()];
    for (e_idx, e) in edges.iter().enumerate() {
        out_edges[e.from].push(e_idx);
        in_edges[e.to].push(e_idx);
    }
    // フロー保存則: Σ f^g_{(v→u)} - Σ f^g_{(u→v)} = a^g_u - 4r^g_u ∀u∈U_g
    for u in 0..blocks.len() {
        let sum_in: Expression = in_edges[u].iter().map(|&ei| f_vars[ei]).sum();
        let sum_out: Expression = out_edges[u].iter().map(|&ei| f_vars[ei]).sum();
        model =
            model.with((sum_in - sum_out).eq(a_vars[u] - REQUIRED_ARROW_BLOCKS_F64 * r_vars[u]));
    }
    // フロー容量制約: 0 ≤ f^g_{(u→v)} ≤ MAX_FLOW_PER_BLOCK*a^g_u ∀(u→v)∈E_g
    for (e_idx, e) in edges.iter().enumerate() {
        model = model.with(f_vars[e_idx] << (MAX_FLOW_PER_BLOCK * a_vars[e.from]));
    }

    // 6) 求解
    let sol = model.solve().map_err(|e| {
        crate::error::KbOptError::SolverError(format!("Failed to solve optimization model: {}", e))
    })?;

    // 7) 解の復元 - 解の情報を直接Geometryに適用
    let objective_ms = sol.eval(&objective_expr);

    // 既存の最適化キーをクリア（固定キーは残す）
    geom.key_placements
        .retain(|_, p| p.placement_type == PlacementType::Fixed);

    // 通常キーの配置を追加
    let _regular_keys_placed = apply_regular_key_placements(geom, &sol, &x_vars, &cands);

    // 矢印キーの配置を追加
    let _arrow_keys_placed = apply_arrow_key_placements(geom, &sol, &m_vars, &blocks);

    Ok(crate::optimize::SolutionLayout { objective_ms })
}

/// 通常キーの配置を適用
fn apply_regular_key_placements(
    geom: &mut Geometry,
    sol: &dyn good_lp::Solution,
    x_vars: &[good_lp::Variable],
    cands: &[Cand],
) -> usize {
    use crate::geometry::types::{KeyPlacement, PlacementType};

    const SOLUTION_THRESHOLD: f64 = 0.5;
    let mut placed = 0;

    for (i, cand) in cands.iter().enumerate() {
        let var_value = sol.value(x_vars[i]);

        if var_value > SOLUTION_THRESHOLD {
            let (x, y) = crate::constants::cell_to_key_center(cand.row, cand.start_col, cand.w_u);

            geom.key_placements.insert(
                cand.key.to_string(),
                KeyPlacement {
                    placement_type: PlacementType::Optimized,
                    key_id: Some(cand.key),
                    x,
                    y,
                    width_u: cand.w_u,
                    block_id: None,
                },
            );
            placed += 1;
        }
    }

    placed
}

/// 矢印キーの配置を適用
fn apply_arrow_key_placements(
    geom: &mut Geometry,
    sol: &dyn good_lp::Solution,
    m_vars: &std::collections::HashMap<(crate::keys::KeyId, usize), good_lp::Variable>,
    blocks: &[Block],
) -> usize {
    use crate::geometry::types::{KeyPlacement, PlacementType};

    const SOLUTION_THRESHOLD: f64 = 0.5;
    let mut placed = 0;

    for &arrow_key in &ARROW_KEYS {
        for (u, block) in blocks.iter().enumerate() {
            let var_value = sol.value(*m_vars.get(&(arrow_key, u)).unwrap());

            if var_value > SOLUTION_THRESHOLD {
                let start_col = block.id.col_u * crate::constants::U2CELL; // 1u = 4 cells
                let (x, y) = crate::constants::cell_to_key_center(block.id.row_u, start_col, 1.0);

                geom.key_placements.insert(
                    arrow_key.to_string(),
                    KeyPlacement {
                        placement_type: PlacementType::Arrow,
                        key_id: Some(arrow_key),
                        x,
                        y,
                        width_u: 1.0,
                        block_id: Some(block.id),
                    },
                );
                placed += 1;
            }
        }
    }
    placed
}

/// v1: 全キーが全行に配置可能な候補を生成
pub fn generate_v1_key_candidates(
    geom: &Geometry,
    movable_keys: &[KeyId],
) -> HashMap<KeyId, KeyCandidates> {
    let free_blocks = extract_free_cell_intervals(geom);
    let mut out = HashMap::new();

    for &k in movable_keys {
        let widths = width_candidates_for_key(&k);
        let mut starts = Vec::new();

        // 全行に配置可能（rは今やu単位の行インデックス）
        for r_u in 0..geom.cells.len() {
            if r_u >= free_blocks.len() {
                continue;
            }
            for &(start, len) in &free_blocks[r_u] {
                for i in start..(start + len) {
                    let mut fits = Vec::new();
                    for &w in &widths {
                        let need = (w * U2CELL as f32).round() as usize;
                        if i + need <= start + len {
                            fits.push(w);
                        }
                    }
                    if !fits.is_empty() {
                        // CellIdのrowはu単位の行インデックス
                        starts.push((CellId::new(r_u, i), fits));
                    }
                }
            }
        }
        out.insert(k, KeyCandidates { starts });
    }
    out
}

/// v1: 全空きセルを矢印キー配置候補とする
pub fn generate_v1_arrow_region(geom: &Geometry) -> (Vec<CellId>, Vec<(CellId, CellId)>) {
    let mut arrow_cells = Vec::new();

    // 全ての空きセルを矢印キー候補に追加
    for r in 0..geom.cells.len() {
        for c in 0..geom.cells[r].len() {
            if !geom.cells[r][c].occupied {
                arrow_cells.push(CellId::new(r, c));
            }
        }
    }

    // 4近傍隣接エッジを生成
    let arrow_set: std::collections::HashSet<_> = arrow_cells.iter().cloned().collect();
    let mut arrow_edges = Vec::new();

    for &cell_id in &arrow_cells {
        let (r, c) = (cell_id.row, cell_id.col);
        let neighbors = [
            (r, c.wrapping_add(1)),
            (r, c.wrapping_sub(1)),
            (r + 1, c),
            (r.wrapping_sub(1), c),
        ];

        for (rr, cc) in neighbors {
            if rr < geom.cells.len() && cc < geom.cells[rr].len() {
                let neighbor_id = CellId::new(rr, cc);
                if arrow_set.contains(&neighbor_id) {
                    arrow_edges.push((cell_id, neighbor_id));
                }
            }
        }
    }

    (arrow_cells, arrow_edges)
}

/// OptimizationSetsから通常キーの候補を生成
pub fn build_candidates_from_precompute(
    geom: &Geometry,
    movable: &BTreeSet<KeyId>,
    optimization_sets: &OptimizationSets,
    opt: &crate::optimize::SolveOptions,
) -> Vec<Cand> {
    let mut out = Vec::new();

    for &key in movable {
        if let Some(key_candidates) = optimization_sets.key_cands.get(&key) {
            for (start_cell, widths) in &key_candidates.starts {
                for &w_u in widths {
                    let w_cells = (w_u * U2CELL as f32).round() as usize;
                    if w_cells == 0 {
                        continue;
                    }

                    // 中心座標計算: 行はu単位、列はcell単位
                    let cx = start_cell.col as f32 / U2CELL as f32 + w_u * 0.5;
                    let cy = start_cell.row as f32; // 既にu単位

                    let finger = geom.cells[start_cell.row][start_cell.col].finger;
                    let home = geom.homes.get(&finger).cloned().unwrap_or((
                        cx * crate::constants::U2MM as f32,
                        cy * crate::constants::U2MM as f32,
                    ));

                    // v1 Fitts計算を使用（key_centerをmm単位に変換）
                    let key_center_mm = (
                        cx * crate::constants::U2MM as f32,
                        cy * crate::constants::U2MM as f32,
                    );
                    let t_ms = compute_key_fitts_time(key_center_mm, home, w_u, opt.a_ms, opt.b_ms);

                    // 新しい座標系: 行はu単位、列はcell単位
                    // キーの物理的境界を正確に計算（列方向のみ、行は揃っているため）
                    let cover_cells: Vec<CellId> = (start_cell.col..start_cell.col + w_cells)
                        .map(|cc| CellId::new(start_cell.row, cc))
                        .collect();

                    out.push(Cand {
                        key,
                        row: start_cell.row,
                        start_col: start_cell.col,
                        w_u,
                        cost_ms: t_ms,
                        cover_cells,
                    });
                }
            }
        }
    }
    out
}

/// OptimizationSetsから矢印用ブロックを生成
pub fn build_blocks_from_precompute(
    _geom: &Geometry,
    optimization_sets: &OptimizationSets,
) -> (Vec<Block>, HashMap<BlockId, usize>) {
    let mut blocks = Vec::new();
    let mut index = HashMap::new();

    // 1uブロック単位でグループ化
    let mut block_cells: HashMap<(usize, usize), Vec<CellId>> = HashMap::new();

    for &cell_id in &optimization_sets.arrow_cells {
        let row = cell_id.row;
        let bcol = cell_id.col / U2CELL;
        block_cells.entry((row, bcol)).or_default().push(cell_id);
    }

    for ((row, bcol), cells) in block_cells {
        if cells.len() == U2CELL {
            // 完全な1uブロックのみ追加（簡略化）
            let start_col = bcol * U2CELL;
            let x0 = start_col as f32 / U2CELL as f32;
            let cx = x0 + 0.5;
            let cy = row as f32;

            let cover_cells = [
                CellId::new(row, start_col),
                CellId::new(row, start_col + 1),
                CellId::new(row, start_col + 2),
                CellId::new(row, start_col + 3),
            ];

            let block_id = BlockId::new(row, bcol);
            let idx = blocks.len();

            blocks.push(Block {
                id: block_id,
                center: (cx, cy),
                cover_cells,
            });
            index.insert(block_id, idx);
        }
    }

    (blocks, index)
}

/// OptimizationSetsから隣接エッジを生成
pub fn build_adjacency_from_precompute(
    blocks: &[Block],
    optimization_sets: &OptimizationSets,
) -> Vec<(usize, usize)> {
    let mut block_index: HashMap<CellId, usize> = HashMap::new();
    for (i, block) in blocks.iter().enumerate() {
        for &cell_id in &block.cover_cells {
            block_index.insert(cell_id, i);
        }
    }

    let mut edges = Vec::new();
    for &(from_cell, to_cell) in &optimization_sets.arrow_edges {
        if let (Some(&from_block), Some(&to_block)) =
            (block_index.get(&from_cell), block_index.get(&to_cell))
            && from_block != to_block
        {
            edges.push((from_block, to_block));
        }
    }

    edges
}
