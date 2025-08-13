use std::collections::{BTreeSet, HashMap};

use good_lp::{Expression, ProblemVariables, Solution, SolverModel, Variable, coin_cbc, variable};

use crate::constants::U2MM;
use crate::csv_reader::KeyFreq;
use crate::error::KbOptError;
use crate::geometry::{Geometry, types::*};
use crate::keys::{ArrowKey, KeyId};

/// ソルバ設定・Fitts 係数など
#[derive(Debug, Clone)]
pub struct SolveOptions {
    pub include_fkeys: bool, // F1..F12 を動かすか
    pub a_ms: f64,           // Fitts: a
    pub b_ms: f64,           // Fitts: b
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            include_fkeys: false,
            a_ms: 0.0,
            b_ms: 1.0,
        }
    }
}

/// 矢印キー定数
const ARROW_KEYS: [KeyId; 4] = [
    KeyId::Arrow(ArrowKey::Up),
    KeyId::Arrow(ArrowKey::Down),
    KeyId::Arrow(ArrowKey::Left),
    KeyId::Arrow(ArrowKey::Right),
];

/// キー種別判定
fn is_arrow(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Arrow(_))
}

fn is_digit_or_f(key_id: &KeyId) -> bool {
    matches!(key_id, KeyId::Digit(_) | KeyId::Function(_))
}

/// 幅候補（0.25u 刻み）。数字/F/矢印は 1u 固定。
fn width_candidates_for_key(key_id: &KeyId) -> Vec<f32> {
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
struct Cand {
    key: KeyId,
    row: usize,
    start_col: usize, // 0.25u index
    w_u: f32,
    cost_ms: f64, // f_k を掛ける前の素コスト
    cover_cells: Vec<CellId>,
}

/// 1u ブロック（矢印用の占有単位）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockId {
    pub row: usize,
    pub bcol: usize, // 1u ブロック列（0.25u 4セルごと）
}
#[derive(Debug, Clone)]
struct Block {
    id: BlockId,
    center: (f32, f32),       // [u]
    cover_cells: [CellId; 4], // この1uが覆う 0.25u セル
}

/// 解の保持
#[derive(Debug, Clone)]
pub struct SolutionLayout {
    pub objective_ms: f64,
    // key -> (row, start_col(0.25u), w_u)
    pub key_place: HashMap<String, (usize, usize, f32)>,
    // arrows: name -> BlockId
    pub arrow_place: HashMap<String, BlockId>,
}

pub fn solve_layout(
    geom: &Geometry,
    freqs: &KeyFreq,
    opt: &SolveOptions,
) -> Result<SolutionLayout, KbOptError> {
    // 1) 集合を作る
    let mut movable: BTreeSet<KeyId> = freqs
        .counts()
        .keys()
        .filter(|k| !is_arrow(k))
        .cloned()
        .collect();

    // Fキーの有無
    if !opt.include_fkeys {
        movable.retain(|k| !matches!(k, KeyId::Function(_)));
    }

    // 2) 通常キーの候補を生成
    let cands = build_candidates(geom, &movable, opt);

    // 3) 矢印用 1u ブロックと隣接グラフ
    let (blocks, _block_index) = build_blocks_1u(geom);
    let adj_edges = build_block_adjacency(&blocks, geom);

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

    // 目的関数: Σ f_k·T^g(j,w)·x^g_{k,j,w} + Σ f_a·T^g_arrow(u)·m^g_{a,u}
    let mut obj = Expression::from(0.0);
    // 通常キー項: Σ_{k∈K} Σ_{j∈I^g_k} Σ_{w∈W_k} f_k·T^g(j,w)·x^g_{k,j,w}
    for (i, cand) in cands.iter().enumerate() {
        let f_k = freqs.get_count(cand.key) as f64;
        obj += f_k * cand.cost_ms * x_vars[i];
    }
    // 矢印キー項: Σ_{a∈A} Σ_{u∈U^g} f_a·T^g_arrow(u)·m^g_{a,u}
    for (u, blk) in blocks.iter().enumerate() {
        let center_cell = blk.cover_cells[2]; // 中央近傍
        let finger = geom.cells[center_cell.row][center_cell.col].finger;
        let home = geom
            .homes
            .get(&finger)
            .cloned()
            .unwrap_or((blk.center.0, blk.center.1));
        let d_u = euclid_u(blk.center, home) as f64 * U2MM;
        let w_mm = 1.0f64 * U2MM;
        let t_ms = opt.a_ms + opt.b_ms * ((d_u / w_mm + 1.0).log2());
        for &arrow_key in &ARROW_KEYS {
            let f_a = freqs.get_count(arrow_key) as f64;
            if f_a > 0.0 {
                let m_au = m_vars.get(&(arrow_key, u)).unwrap();
                obj += (f_a * t_ms) * *m_au;
            }
        }
    }

    // 目的関数を後で評価するために保存
    let objective_expr = obj.clone();

    // 5) 制約条件

    let mut model = vars.minimise(obj).using(coin_cbc);

    // (i) 一意性制約: Σ_{j∈I^g_k} Σ_{w∈W_k} x^g_{k,j,w} = 1 ∀k∈K
    for &key in movable.iter() {
        let idxs: Vec<usize> = cands
            .iter()
            .enumerate()
            .filter(|(_, c)| c.key == key)
            .map(|(i, _)| i)
            .collect();
        if !idxs.is_empty() {
            let sum: Expression = idxs.iter().map(|i| x_vars[*i]).sum();
            // 頻度>0のキーは必須配置
            if freqs.get_count(key) > 0 {
                model = model.with(sum.clone().eq(1));
            } else {
                model = model.with(sum.clone() << 1);
            }
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
    for r in 0..geom.cfg.rows.len() {
        for c in 0..geom.cells_per_row {
            let cid = CellId::new(r, c);
            let fixed = if geom.cells[r][c].fixed_occupied {
                1.0
            } else {
                0.0
            };
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
    // (v) 矢印キー総数制約: Σ_{u∈U_g} a^g_u = 4
    let sum_a_total: Expression = (0..blocks.len()).map(|u| a_vars[u]).sum();
    model = model.with(sum_a_total.eq(4));

    // (vi) 矢印キー連結制約（フロー保存）
    // フロー根一意性: Σ_{u∈U_g} r^g_u = 1
    let sum_r: Expression = (0..blocks.len()).map(|u| r_vars[u]).sum();
    model = model.with(sum_r.eq(1));

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
        model = model.with((sum_in - sum_out).eq(a_vars[u] - 4 * r_vars[u]));
    }
    // フロー容量制約: 0 ≤ f^g_{(u→v)} ≤ 3a^g_u ∀(u→v)∈E_g
    for (e_idx, e) in edges.iter().enumerate() {
        model = model.with(f_vars[e_idx] << (3.0 * a_vars[e.from]));
    }

    // 6) 求解
    let sol = model
        .solve()
        .map_err(|e| KbOptError::Other(e.to_string()))?;

    // 7) 解の復元
    let mut key_place = HashMap::new();
    for (i, cand) in cands.iter().enumerate() {
        if sol.value(x_vars[i]) > 0.5 {
            key_place.insert(cand.key.to_string(), (cand.row, cand.start_col, cand.w_u));
        }
    }
    let mut arrow_place = HashMap::new();
    for &arrow_key in &ARROW_KEYS {
        for (u, block) in blocks.iter().enumerate() {
            if sol.value(*m_vars.get(&(arrow_key, u)).unwrap()) > 0.5 {
                arrow_place.insert(arrow_key.to_string(), block.id);
            }
        }
    }
    let objective_ms = sol.eval(&objective_expr);

    Ok(SolutionLayout {
        objective_ms,
        key_place,
        arrow_place,
    })
}

/* ----------------- 内部：候補生成・ブロック構築 ----------------- */

fn build_candidates(geom: &Geometry, movable: &BTreeSet<KeyId>, opt: &SolveOptions) -> Vec<Cand> {
    let mut out = Vec::new();
    for &key in movable {
        let widths = width_candidates_for_key(&key);
        let fk_dummy = 1.0f64; // ここでは素コスト（頻度は後で掛ける）
        for r in 0..geom.cfg.rows.len() {
            let row = &geom.cfg.rows[r];
            for &w_u in &widths {
                let w_cells = cells_from_u(w_u);
                if w_cells == 0 {
                    continue;
                }
                for c0 in 0..=geom.cells_per_row - w_cells {
                    // 固定占有に当たらないかチェック
                    if (c0..c0 + w_cells).any(|c| geom.cells[r][c].fixed_occupied) {
                        continue;
                    }
                    // 中心セルの指でホームを取る
                    let c_center = c0 + w_cells / 2;
                    let (cx, cy) = {
                        let x0 = row.offset_u + c0 as f32 * CELL_U;
                        let x1 = row.offset_u + (c0 + w_cells) as f32 * CELL_U;
                        ((x0 + x1) * 0.5, row.base_y_u)
                    };
                    let finger = geom.cells[r][c_center].finger;
                    let home = geom.homes.get(&finger).cloned().unwrap_or((cx, cy));
                    let d_mm = (euclid_u((cx, cy), home) as f64) * U2MM;
                    let w_mm = (w_u as f64) * U2MM;
                    let t_ms = opt.a_ms + opt.b_ms * ((d_mm / w_mm + 1.0).log2());
                    let cover_cells: Vec<CellId> =
                        (c0..c0 + w_cells).map(|cc| CellId::new(r, cc)).collect();
                    out.push(Cand {
                        key,
                        row: r,
                        start_col: c0,
                        w_u,
                        cost_ms: t_ms * fk_dummy,
                        cover_cells,
                    });
                }
            }
        }
    }
    out
}

fn euclid_u(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}

fn build_blocks_1u(geom: &Geometry) -> (Vec<Block>, HashMap<BlockId, usize>) {
    let mut blocks = Vec::new();
    let mut index = HashMap::new();
    for r in 0..geom.cfg.rows.len() {
        let row = &geom.cfg.rows[r];
        let bcols = geom.cells_per_row / cells_from_u(ONE_U);
        for b in 0..bcols {
            let start_col = b * cells_from_u(ONE_U);
            // 固定占有に当たらないか
            if (start_col..start_col + cells_from_u(ONE_U)).any(|c| geom.cells[r][c].fixed_occupied)
            {
                continue;
            }
            let x0 = row.offset_u + start_col as f32 * CELL_U;
            let cx = x0 + 0.5 * ONE_U;
            let cy = row.base_y_u;
            let ids = [
                CellId::new(r, start_col),
                CellId::new(r, start_col + 1),
                CellId::new(r, start_col + 2),
                CellId::new(r, start_col + 3),
            ];
            let id = BlockId { row: r, bcol: b };
            let idx = blocks.len();
            blocks.push(Block {
                id,
                center: (cx, cy),
                cover_cells: ids,
            });
            index.insert(id, idx);
        }
    }
    (blocks, index)
}

/// 1u ブロックの隣接（8近傍：水平/垂直/斜め いずれかが接する）
fn build_block_adjacency(blocks: &[Block], geom: &Geometry) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for (i, bi) in blocks.iter().enumerate() {
        for (j, bj) in blocks.iter().enumerate().skip(i + 1) {
            if blocks_touch(bi, bj, geom) {
                edges.push((i, j));
                edges.push((j, i));
            }
        }
    }
    edges
}
fn blocks_touch(a: &Block, b: &Block, geom: &Geometry) -> bool {
    // a,b の 1u 矩形が辺または角で接していれば true
    let rect = |blk: &Block| {
        // 各セルは 0.25u 正方形。ブロックは横1u, 縦1u
        let r = blk.cover_cells[0].row;
        let c0 = blk.cover_cells[0].col;
        let x0 = geom.cfg.rows[r].offset_u + c0 as f32 * CELL_U;
        let y0 = geom.cfg.rows[r].base_y_u - 0.5;
        (x0, y0, x0 + ONE_U, y0 + 1.0)
    };
    let (ax0, ay0, ax1, ay1) = rect(a);
    let (bx0, by0, bx1, by1) = rect(b);
    let sep_x = (ax1 < bx0) || (bx1 < ax0);
    let sep_y = (ay1 < by0) || (by1 < ay0);
    if sep_x || sep_y {
        // 角接触（端点一致）も許す：距離ゼロなら true
        let dx = if ax1 < bx0 {
            bx0 - ax1
        } else if bx1 < ax0 {
            ax0 - bx1
        } else {
            0.0
        };
        let dy = if ay1 < by0 {
            by0 - ay1
        } else if by1 < ay0 {
            ay0 - by1
        } else {
            0.0
        };
        (dx.abs() <= 1e-6) && (dy.abs() <= 1e-6)
    } else {
        true // 矩形が重なる or 辺接触
    }
}
