//! DSL: grid -> grid operations, some with a colour parameter. v0 first-order; v1 adds object, line, symmetry and
//! crop operations; v1 part 2 (E16) adds fixed templates over objects (select, per-object transforms, rigid moves).
//! Each operation has a cost in bits; search tries cheaper programs first.
use crate::grid::Grid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    FlipH, FlipV, Rot90, Rot180, Rot270, Transpose, AntiTranspose,
    CropContent, CropLargest, CropSmallest, CropColour(u8),
    Up2, Up3, Down2, Down3,
    TileH, TileV, Tile2x2, MirrorH, MirrorV, Mirror4,
    Left, Right, Top, Bottom,
    AndLR(u8), OrLR(u8), XorLR(u8), NorLR(u8), AndTB(u8), OrTB(u8), XorTB(u8), NorTB(u8),
    Recolour(u8, u8), Keep(u8), Remove(u8), FillBg(u8),
    FallDown, FallUp, FallLeft, FallRight,
    Dedup, StripBgLines, FillEnclosed(u8), OutlineObjects(u8),
    // DSL v1
    BoxObjects(u8), FillBoxes(u8),
    ShootUp(u8), ShootDown(u8), ShootLeft(u8), ShootRight(u8), Connect(u8),
    SymH, SymV, Sym4, SymD, CropInside(u8), SelfKron,
    // DSL v1 part 2 (E16): fixed higher-order templates over objects
    Sel(Act, Parse, Pred), EachObject(Body), Move(Dir),
    // DSL v2 batch 1 (E24): from the train-pool gap analysis (Hodel solutions of unsolved train tasks)
    DownSample2, DownSample3, Period, UpByColours, UpByCells, FillInbox(u8),
    BoxAround(u8, u8), HaloOrth(u8, u8), HaloDiag(u8, u8), LinesV(u8, u8), LinesH(u8, u8), ConnectWith(u8, u8),
    ShootOwn(Dir), CountRow, CountObjRow, ColourOfLargest, Rot4Tile,
}
use Op::*;

/// How a grid is cut into objects. Multi: 4-connected non-background cells of any colours. Mono: 4-connected cells of
/// one non-background colour. Diag: like Mono, but 8-connected (cells touching at corners join).
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Parse { Multi, Mono, Diag }
/// Which objects a selection picks. Largest/Smallest need a unique extreme; Odd* pick the objects whose shape, colour
/// or size no other object shares (needs at least 3 objects).
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Pred { Largest, Smallest, Single, Border, Inner, OddShape, OddColour, OddSize, Holed, Solid }
/// What a selection does with the picked objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Act { Keep, Remove, Crop, Recolour(u8) }
/// An in-place transform applied to every object within its own bounding box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Body { FlipH, FlipV, Rot180 }
/// Rigid move of every object until it meets the border or another object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Dir { Up, Down, Left, Right }
const PREDS: [Pred; 10] = [Pred::Largest, Pred::Smallest, Pred::Single, Pred::Border, Pred::Inner, Pred::OddShape, Pred::OddColour, Pred::OddSize, Pred::Holed, Pred::Solid];

impl Op {
    pub fn cost(self) -> u16 {
        match self {
            FlipH | FlipV | Rot90 | Rot180 | Rot270 | Transpose | AntiTranspose => 3,
            CropContent | CropLargest | CropSmallest | Up2 | Up3 | Down2 | Down3 | TileH | TileV | Tile2x2
            | MirrorH | MirrorV | Mirror4 | Left | Right | Top | Bottom | FallDown | FallUp | FallLeft | FallRight
            | Dedup | StripBgLines => 4,
            CropColour(_) | Keep(_) | Remove(_) | FillBg(_) | FillEnclosed(_) | OutlineObjects(_) => 6,
            AndLR(_) | OrLR(_) | XorLR(_) | NorLR(_) | AndTB(_) | OrTB(_) | XorTB(_) | NorTB(_) => 7,
            Recolour(_, _) => 8,
            SymH | SymV | Sym4 | SymD => 4,
            SelfKron => 5,
            BoxObjects(_) | FillBoxes(_)
            | ShootUp(_) | ShootDown(_) | ShootLeft(_) | ShootRight(_) | Connect(_) | CropInside(_) => 6,
            Sel(Act::Recolour(_), _, _) => 7,
            Sel(_, _, _) => 6,
            EachObject(_) | Move(_) => 5,
            DownSample2 | DownSample3 | Period | UpByColours | UpByCells | ShootOwn(_) | CountRow | CountObjRow | ColourOfLargest | Rot4Tile => 5,
            FillInbox(_) => 6,
            BoxAround(_, _) | HaloOrth(_, _) | HaloDiag(_, _) | LinesV(_, _) | LinesH(_, _) | ConnectWith(_, _) => 8,
        }
    }
    pub fn name(self) -> String { format!("{:?}", self) }
    /// The operation's kind for the prior (E17): its name without colour arguments; Sel keeps action, parsing and
    /// predicate, e.g. Sel(Recolour,Multi,Holed). Must match tools/make_prior.py.
    pub fn kind(self) -> String {
        match self {
            Sel(act, parse, pred) => {
                let a = match act { Act::Keep => "Keep", Act::Remove => "Remove", Act::Crop => "Crop", Act::Recolour(_) => "Recolour" };
                format!("Sel({},{:?},{:?})", a, parse, pred)
            }
            _ => { let n = self.name(); n.split('(').next().unwrap_or("").to_string() }
        }
    }
}

/// All operations to try for a task, given the colours that appear in it.
pub fn ops_for(palette: &[u8]) -> Vec<Op> {
    let mut v = vec![FlipH, FlipV, Rot90, Rot180, Rot270, Transpose, AntiTranspose, CropContent, CropLargest, CropSmallest,
        Up2, Up3, Down2, Down3, TileH, TileV, Tile2x2, MirrorH, MirrorV, Mirror4, Left, Right, Top, Bottom,
        FallDown, FallUp, FallLeft, FallRight, Dedup, StripBgLines,
        SymH, SymV, Sym4, SymD, SelfKron,
        EachObject(Body::FlipH), EachObject(Body::FlipV), EachObject(Body::Rot180),
        Move(Dir::Up), Move(Dir::Down), Move(Dir::Left), Move(Dir::Right),
        DownSample2, DownSample3, Period, UpByColours, UpByCells, CountRow, CountObjRow, ColourOfLargest, Rot4Tile,
        ShootOwn(Dir::Up), ShootOwn(Dir::Down), ShootOwn(Dir::Left), ShootOwn(Dir::Right)];
    for &c in palette { v.push(FillInbox(c)); }
    for &a in palette { for &c in palette { if a != c {
        v.extend([BoxAround(a, c), HaloOrth(a, c), HaloDiag(a, c), LinesV(a, c), LinesH(a, c), ConnectWith(a, c)]);
    } } }
    for parse in [Parse::Multi, Parse::Mono, Parse::Diag] {
        for pred in PREDS {
            for act in [Act::Keep, Act::Remove, Act::Crop] { v.push(Sel(act, parse, pred)); }
            for &c in palette { v.push(Sel(Act::Recolour(c), parse, pred)); }
        }
    }
    for &c in palette {
        v.extend([CropColour(c), Keep(c), Remove(c), FillBg(c), FillEnclosed(c), OutlineObjects(c),
            AndLR(c), OrLR(c), XorLR(c), NorLR(c), AndTB(c), OrTB(c), XorTB(c), NorTB(c),
            BoxObjects(c), FillBoxes(c),
            ShootUp(c), ShootDown(c), ShootLeft(c), ShootRight(c), Connect(c), CropInside(c)]);
        for &d in palette { if d != c { v.push(Recolour(c, d)); } }
    }
    v.sort_by_key(|o| o.cost());
    v
}

fn map(g: &Grid, h: usize, w: usize, f: impl Fn(usize, usize) -> u8) -> Grid {
    let mut o = Grid::new(h, w, 0);
    for r in 0..h { for k in 0..w { o.set(r, k, f(r, k)); } }
    o
}

fn crop(g: &Grid, r0: usize, k0: usize, r1: usize, k1: usize) -> Option<Grid> {
    if r1 < r0 || k1 < k0 { return None; }
    Some(map(g, r1 - r0 + 1, k1 - k0 + 1, |r, k| g.at(r0 + r, k0 + k)))
}

fn bbox(g: &Grid, keep: impl Fn(u8) -> bool) -> Option<(usize, usize, usize, usize)> {
    let (mut r0, mut k0, mut r1, mut k1) = (usize::MAX, usize::MAX, 0, 0);
    for r in 0..g.h { for k in 0..g.w { if keep(g.at(r, k)) { r0 = r0.min(r); k0 = k0.min(k); r1 = r1.max(r); k1 = k1.max(k); } } }
    if r0 == usize::MAX { None } else { Some((r0, k0, r1, k1)) }
}

/// Connected groups of non-background cells (4-neighbour, any colours), as cell lists.
fn objects(g: &Grid) -> Vec<Vec<(usize, usize)>> {
    let bg = g.bg();
    let mut seen = vec![false; g.h * g.w];
    let mut out = Vec::new();
    for s in 0..g.h * g.w {
        if seen[s] || g.c[s] == bg { continue; }
        let mut stack = vec![s]; seen[s] = true; let mut cells = Vec::new();
        while let Some(p) = stack.pop() {
            let (r, k) = (p / g.w, p % g.w); cells.push((r, k));
            let mut nb = Vec::with_capacity(4);
            if r > 0 { nb.push(p - g.w); } if r + 1 < g.h { nb.push(p + g.w); }
            if k > 0 { nb.push(p - 1); } if k + 1 < g.w { nb.push(p + 1); }
            for q in nb { if !seen[q] && g.c[q] != bg { seen[q] = true; stack.push(q); } }
        }
        out.push(cells);
    }
    out
}

fn crop_cells(g: &Grid, cells: &[(usize, usize)]) -> Option<Grid> {
    let r0 = cells.iter().map(|c| c.0).min()?; let r1 = cells.iter().map(|c| c.0).max()?;
    let k0 = cells.iter().map(|c| c.1).min()?; let k1 = cells.iter().map(|c| c.1).max()?;
    crop(g, r0, k0, r1, k1)
}

/// Split into two halves left/right (or top/bottom), skipping a one-cell separator line if present.
fn halves(g: &Grid, lr: bool) -> Option<(Grid, Grid)> {
    let (n, other) = if lr { (g.w, g.h) } else { (g.h, g.w) };
    let line_uniform = |i: usize| { let v = if lr { g.at(0, i) } else { g.at(i, 0) }; (0..other).all(|j| (if lr { g.at(j, i) } else { g.at(i, j) }) == v) };
    let (a_end, b_start) = if n % 2 == 1 && n >= 3 && line_uniform(n / 2) { (n / 2, n / 2 + 1) } else if n % 2 == 0 && n >= 2 { (n / 2, n / 2) } else { return None };
    let len = a_end;
    if lr { Some((crop(g, 0, 0, g.h - 1, len - 1)?, crop(g, 0, b_start, g.h - 1, b_start + len - 1)?)) }
    else { Some((crop(g, 0, 0, len - 1, g.w - 1)?, crop(g, b_start, 0, b_start + len - 1, g.w - 1)?)) }
}

fn combine(g: &Grid, lr: bool, c: u8, f: impl Fn(bool, bool) -> bool) -> Option<Grid> {
    let (a, b) = halves(g, lr)?;
    let bg = g.bg();   // the whole grid's background: a half can be mostly coloured
    let (ba, bb) = (bg, bg);
    Some(map(&a, a.h, a.w, |r, k| if f(a.at(r, k) != ba, b.at(r, k) != bb) { c } else { 0 }))
}

fn fall(g: &Grid, dr: i32, dk: i32) -> Grid {
    let bg = g.bg();
    let mut o = Grid::new(g.h, g.w, bg);
    if dk == 0 {
        for k in 0..g.w {
            let col: Vec<u8> = (0..g.h).map(|r| g.at(r, k)).filter(|&v| v != bg).collect();
            for (i, &v) in col.iter().enumerate() { let r = if dr > 0 { g.h - col.len() + i } else { i }; o.set(r, k, v); }
        }
    } else {
        for r in 0..g.h {
            let row: Vec<u8> = (0..g.w).map(|k| g.at(r, k)).filter(|&v| v != bg).collect();
            for (i, &v) in row.iter().enumerate() { let k = if dk > 0 { g.w - row.len() + i } else { i }; o.set(r, k, v); }
        }
    }
    o
}

fn paint_cells(g: &Grid, cells: &[(usize, usize)], c: u8) -> Grid { let mut o = g.clone(); for &(r, k) in cells { o.set(r, k, c); } o }
fn clear_cells(g: &Grid, cells: &[(usize, usize)]) -> Grid { let bg = g.bg(); paint_cells(g, cells, bg) }
fn only_cells(g: &Grid, cells: &[(usize, usize)]) -> Grid { let mut o = Grid::new(g.h, g.w, g.bg()); for &(r, k) in cells { o.set(r, k, g.at(r, k)); } o }
fn shoot(g: &Grid, c: u8, dr: i32, dk: i32) -> Option<Grid> {
    let bg = g.bg(); if c == bg || g.colours() & (1 << c) == 0 { return None; }
    let mut o = g.clone(); let mut any = false;
    for r in 0..g.h { for k in 0..g.w {
        if g.at(r, k) != c { continue; }
        let (mut rr, mut kk) = (r as i32 + dr, k as i32 + dk);
        while rr >= 0 && kk >= 0 && (rr as usize) < g.h && (kk as usize) < g.w && g.at(rr as usize, kk as usize) == bg {
            o.set(rr as usize, kk as usize, c); any = true; rr += dr; kk += dk;
        }
    } }
    if any { Some(o) } else { None }
}
fn symmetrize(g: &Grid, h: bool, v: bool, d: bool) -> Option<Grid> {
    if d && g.h != g.w { return None; }
    let bg = g.bg(); let mut o = g.clone(); let mut changed = true; let mut any = false;
    while changed {
        changed = false;
        for r in 0..g.h { for k in 0..g.w {
            if o.at(r, k) != bg { continue; }
            let mut cand = Vec::new();
            if h { cand.push((r, g.w - 1 - k)); }
            if v { cand.push((g.h - 1 - r, k)); }
            if h && v { cand.push((g.h - 1 - r, g.w - 1 - k)); }
            if d { cand.push((k, r)); }
            for (rr, kk) in cand { let x = o.at(rr, kk); if x != bg { o.set(r, k, x); changed = true; any = true; break; } }
        } }
    }
    if any { Some(o) } else { None }
}

/// Objects under a parsing (see `Parse`).
fn objects_by(g: &Grid, parse: Parse) -> Vec<Vec<(usize, usize)>> {
    if parse == Parse::Multi { return objects(g); }
    let diag = parse == Parse::Diag;
    let bg = g.bg();
    let mut seen = vec![false; g.h * g.w];
    let mut out = Vec::new();
    for s in 0..g.h * g.w {
        if seen[s] || g.c[s] == bg { continue; }
        let col = g.c[s];
        let mut stack = vec![s]; seen[s] = true; let mut cells = Vec::new();
        while let Some(p) = stack.pop() {
            let (r, k) = (p / g.w, p % g.w); cells.push((r, k));
            let mut nb = Vec::with_capacity(4);
            if r > 0 { nb.push(p - g.w); } if r + 1 < g.h { nb.push(p + g.w); }
            if k > 0 { nb.push(p - 1); } if k + 1 < g.w { nb.push(p + 1); }
            if diag {
                if r > 0 && k > 0 { nb.push(p - g.w - 1); } if r > 0 && k + 1 < g.w { nb.push(p - g.w + 1); }
                if r + 1 < g.h && k > 0 { nb.push(p + g.w - 1); } if r + 1 < g.h && k + 1 < g.w { nb.push(p + g.w + 1); }
            }
            for q in nb { if !seen[q] && g.c[q] == col { seen[q] = true; stack.push(q); } }
        }
        out.push(cells);
    }
    out
}

fn shape_key(cells: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let r0 = cells.iter().map(|c| c.0).min().unwrap_or(0); let k0 = cells.iter().map(|c| c.1).min().unwrap_or(0);
    let mut v: Vec<(usize, usize)> = cells.iter().map(|&(r, k)| (r - r0, k - k0)).collect(); v.sort(); v
}
fn shape_hash(cells: &[(usize, usize)]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new(); shape_key(cells).hash(&mut h); h.finish()
}
fn colour_key(g: &Grid, cells: &[(usize, usize)]) -> u16 { let mut m = 0u16; for &(r, k) in cells { m |= 1 << g.at(r, k); } m }
/// Does the object enclose cells that are not its own (a hole), within its bounding box?
fn has_hole(cells: &[(usize, usize)]) -> bool {
    let r0 = cells.iter().map(|c| c.0).min().unwrap(); let r1 = cells.iter().map(|c| c.0).max().unwrap();
    let k0 = cells.iter().map(|c| c.1).min().unwrap(); let k1 = cells.iter().map(|c| c.1).max().unwrap();
    let (h, w) = (r1 - r0 + 3, k1 - k0 + 3);
    let mut own = vec![false; h * w];
    for &(r, k) in cells { own[(r - r0 + 1) * w + (k - k0 + 1)] = true; }
    let mut out = vec![false; h * w]; out[0] = true; let mut stack = vec![0usize];
    while let Some(p) = stack.pop() {
        let (r, k) = (p / w, p % w);
        let mut nb = Vec::with_capacity(4);
        if r > 0 { nb.push(p - w); } if r + 1 < h { nb.push(p + w); } if k > 0 { nb.push(p - 1); } if k + 1 < w { nb.push(p + 1); }
        for q in nb { if !out[q] && !own[q] { out[q] = true; stack.push(q); } }
    }
    (0..h * w).any(|p| !own[p] && !out[p])
}
/// Indices of the objects a predicate picks; None when the pick is ambiguous or empty.
fn pick(g: &Grid, objs: &[Vec<(usize, usize)>], pred: Pred) -> Option<Vec<usize>> {
    let n = objs.len();
    if n < 2 { return None; }
    let odd = |keys: Vec<u64>| -> Option<Vec<usize>> {
        if n < 3 { return None; }
        let v: Vec<usize> = (0..n).filter(|&i| keys.iter().filter(|&&x| x == keys[i]).count() == 1).collect();
        if v.len() == 1 { Some(v) } else { None }
    };
    let v: Vec<usize> = match pred {
        Pred::Largest | Pred::Smallest => {
            let best = if pred == Pred::Largest { objs.iter().map(|o| o.len()).max()? } else { objs.iter().map(|o| o.len()).min()? };
            let v: Vec<usize> = (0..n).filter(|&i| objs[i].len() == best).collect();
            if v.len() != 1 { return None; } v
        }
        Pred::Single => (0..n).filter(|&i| objs[i].len() == 1).collect(),
        Pred::Border | Pred::Inner => {
            let edge = |o: &Vec<(usize, usize)>| o.iter().any(|&(r, k)| r == 0 || k == 0 || r + 1 == g.h || k + 1 == g.w);
            (0..n).filter(|&i| edge(&objs[i]) == (pred == Pred::Border)).collect()
        }
        Pred::OddShape => return odd(objs.iter().map(|o| shape_hash(o)).collect()),
        Pred::OddColour => return odd(objs.iter().map(|o| colour_key(g, o) as u64).collect()),
        Pred::OddSize => return odd(objs.iter().map(|o| o.len() as u64).collect()),
        Pred::Holed | Pred::Solid => (0..n).filter(|&i| has_hole(&objs[i]) == (pred == Pred::Holed)).collect(),
    };
    if v.is_empty() || v.len() == n { None } else { Some(v) }
}
fn select(g: &Grid, act: Act, parse: Parse, pred: Pred) -> Option<Grid> {
    let objs = objects_by(g, parse);
    let idx = pick(g, &objs, pred)?;
    let cells: Vec<(usize, usize)> = idx.iter().flat_map(|&i| objs[i].iter().copied()).collect();
    Some(match act {
        Act::Keep => only_cells(g, &cells),
        Act::Remove => clear_cells(g, &cells),
        Act::Crop => { if idx.len() != 1 { return None; } crop_cells(g, &objs[idx[0]])? }
        Act::Recolour(c) => { if cells.iter().all(|&(r, k)| g.at(r, k) == c) { return None; } paint_cells(g, &cells, c) }
    })
}
fn each_object(g: &Grid, body: Body) -> Option<Grid> {
    let objs = objects(g); if objs.is_empty() { return None; }
    let bg = g.bg(); let mut o = g.clone();
    for cells in &objs { for &(r, k) in cells { o.set(r, k, bg); } }
    for cells in &objs {
        let r0 = cells.iter().map(|c| c.0).min()?; let r1 = cells.iter().map(|c| c.0).max()?;
        let k0 = cells.iter().map(|c| c.1).min()?; let k1 = cells.iter().map(|c| c.1).max()?;
        for &(r, k) in cells {
            let (rr, kk) = match body { Body::FlipH => (r, k0 + k1 - k), Body::FlipV => (r0 + r1 - r, k), Body::Rot180 => (r0 + r1 - r, k0 + k1 - k) };
            o.set(rr, kk, g.at(r, k));
        }
    }
    if o == *g { None } else { Some(o) }
}
fn move_objects(g: &Grid, dir: Dir) -> Option<Grid> {
    let mut objs = objects(g); if objs.is_empty() { return None; }
    let bg = g.bg();
    let (dr, dk): (i32, i32) = match dir { Dir::Up => (-1, 0), Dir::Down => (1, 0), Dir::Left => (0, -1), Dir::Right => (0, 1) };
    // objects nearest the target side move first
    let lead = |o: &Vec<(usize, usize)>| -> i32 { o.iter().map(|&(r, k)| r as i32 * dr + k as i32 * dk).max().unwrap() };
    objs.sort_by_key(|o| -lead(o));
    let mut o = g.clone();
    for cells in &objs {
        let colours: Vec<u8> = cells.iter().map(|&(r, k)| o.at(r, k)).collect();
        for &(r, k) in cells { o.set(r, k, bg); }
        let mut step = 0i32;
        loop {
            let s = step + 1;
            let ok = cells.iter().all(|&(r, k)| {
                let (rr, kk) = (r as i32 + dr * s, k as i32 + dk * s);
                rr >= 0 && kk >= 0 && (rr as usize) < g.h && (kk as usize) < g.w && o.at(rr as usize, kk as usize) == bg
            });
            if !ok { break; } step = s;
        }
        for (i, &(r, k)) in cells.iter().enumerate() { o.set((r as i32 + dr * step) as usize, (k as i32 + dk * step) as usize, colours[i]); }
    }
    if o == *g { None } else { Some(o) }
}

// ---- DSL v2 batch 1 helpers (E24) ----
fn bbox_of(cells: &[(usize, usize)]) -> (usize, usize, usize, usize) {
    let r0 = cells.iter().map(|c| c.0).min().unwrap(); let r1 = cells.iter().map(|c| c.0).max().unwrap();
    let k0 = cells.iter().map(|c| c.1).min().unwrap(); let k1 = cells.iter().map(|c| c.1).max().unwrap();
    (r0, k0, r1, k1)
}
fn has_colour(g: &Grid, a: u8) -> bool { g.colours() & (1 << a) != 0 }
/// Paint colour c on background cells at the given offsets from every cell of colour a.
fn halo(g: &Grid, a: u8, c: u8, offs: &[(i32, i32)]) -> Option<Grid> {
    let bg = g.bg(); if a == bg || c == bg || !has_colour(g, a) { return None; }
    let mut o = g.clone(); let mut any = false;
    for r in 0..g.h { for k in 0..g.w {
        if g.at(r, k) != a { continue; }
        for &(dr, dk) in offs {
            let (rr, kk) = (r as i32 + dr, k as i32 + dk);
            if rr >= 0 && kk >= 0 && (rr as usize) < g.h && (kk as usize) < g.w && g.at(rr as usize, kk as usize) == bg { o.set(rr as usize, kk as usize, c); any = true; }
        }
    } }
    if any { Some(o) } else { None }
}
fn upscale(g: &Grid, s: usize) -> Option<Grid> {
    if s < 2 || g.h * s > 30 || g.w * s > 30 { return None; }
    Some(map(g, g.h * s, g.w * s, |r, k| g.at(r / s, k / s)))
}
fn main_colour(g: &Grid) -> Option<u8> {
    let bg = g.bg(); let mut n = [0usize; 10]; for &v in &g.c { if v != bg { n[v as usize] += 1; } }
    let (c, cnt) = n.iter().enumerate().max_by_key(|(_, x)| **x)?; if *cnt == 0 { None } else { Some(c as u8) }
}

pub fn apply(op: Op, g: &Grid) -> Option<Grid> {
    let out = match op {
        FlipH => map(g, g.h, g.w, |r, k| g.at(r, g.w - 1 - k)),
        FlipV => map(g, g.h, g.w, |r, k| g.at(g.h - 1 - r, k)),
        Rot90 => map(g, g.w, g.h, |r, k| g.at(g.h - 1 - k, r)),
        Rot180 => map(g, g.h, g.w, |r, k| g.at(g.h - 1 - r, g.w - 1 - k)),
        Rot270 => map(g, g.w, g.h, |r, k| g.at(k, g.w - 1 - r)),
        Transpose => map(g, g.w, g.h, |r, k| g.at(k, r)),
        AntiTranspose => map(g, g.w, g.h, |r, k| g.at(g.h - 1 - k, g.w - 1 - r)),
        CropContent => { let bg = g.bg(); let (r0, k0, r1, k1) = bbox(g, |v| v != bg)?; crop(g, r0, k0, r1, k1)? }
        CropLargest => { let o = objects(g); let best = o.iter().max_by_key(|c| c.len())?; crop_cells(g, best)? }
        CropSmallest => { let o = objects(g); let best = o.iter().min_by_key(|c| c.len())?; crop_cells(g, best)? }
        CropColour(c) => { let (r0, k0, r1, k1) = bbox(g, |v| v == c)?; crop(g, r0, k0, r1, k1)? }
        Up2 | Up3 => { let s = if op == Up2 { 2 } else { 3 }; map(g, g.h * s, g.w * s, |r, k| g.at(r / s, k / s)) }
        Down2 | Down3 => {
            let s = if op == Down2 { 2 } else { 3 };
            if g.h % s != 0 || g.w % s != 0 { return None; }
            let o = map(g, g.h / s, g.w / s, |r, k| g.at(r * s, k * s));
            for r in 0..g.h { for k in 0..g.w { if g.at(r, k) != o.at(r / s, k / s) { return None; } } }
            o
        }
        TileH => map(g, g.h, g.w * 2, |r, k| g.at(r, k % g.w)),
        TileV => map(g, g.h * 2, g.w, |r, k| g.at(r % g.h, k)),
        Tile2x2 => map(g, g.h * 2, g.w * 2, |r, k| g.at(r % g.h, k % g.w)),
        MirrorH => map(g, g.h, g.w * 2, |r, k| if k < g.w { g.at(r, k) } else { g.at(r, 2 * g.w - 1 - k) }),
        MirrorV => map(g, g.h * 2, g.w, |r, k| if r < g.h { g.at(r, k) } else { g.at(2 * g.h - 1 - r, k) }),
        Mirror4 => map(g, g.h * 2, g.w * 2, |r, k| {
            let rr = if r < g.h { r } else { 2 * g.h - 1 - r }; let kk = if k < g.w { k } else { 2 * g.w - 1 - k }; g.at(rr, kk) }),
        Left => halves(g, true)?.0, Right => halves(g, true)?.1, Top => halves(g, false)?.0, Bottom => halves(g, false)?.1,
        AndLR(c) => combine(g, true, c, |a, b| a && b)?, OrLR(c) => combine(g, true, c, |a, b| a || b)?,
        XorLR(c) => combine(g, true, c, |a, b| a != b)?, NorLR(c) => combine(g, true, c, |a, b| !a && !b)?,
        AndTB(c) => combine(g, false, c, |a, b| a && b)?, OrTB(c) => combine(g, false, c, |a, b| a || b)?,
        XorTB(c) => combine(g, false, c, |a, b| a != b)?, NorTB(c) => combine(g, false, c, |a, b| !a && !b)?,
        Recolour(a, b) => { if g.colours() & (1 << a) == 0 { return None; } map(g, g.h, g.w, |r, k| { let v = g.at(r, k); if v == a { b } else { v } }) }
        Keep(c) => { let bg = g.bg(); if c == bg || g.colours() & (1 << c) == 0 { return None; } map(g, g.h, g.w, |r, k| { let v = g.at(r, k); if v == c { v } else { bg } }) }
        Remove(c) => { let bg = g.bg(); if c == bg || g.colours() & (1 << c) == 0 { return None; } map(g, g.h, g.w, |r, k| { let v = g.at(r, k); if v == c { bg } else { v } }) }
        FillBg(c) => { let bg = g.bg(); if c == bg { return None; } map(g, g.h, g.w, |r, k| { let v = g.at(r, k); if v == bg { c } else { v } }) }
        FallDown => fall(g, 1, 0), FallUp => fall(g, -1, 0), FallRight => fall(g, 0, 1), FallLeft => fall(g, 0, -1),
        Dedup => {
            let rows: Vec<usize> = (0..g.h).filter(|&r| r == 0 || (0..g.w).any(|k| g.at(r, k) != g.at(r - 1, k))).collect();
            let cols: Vec<usize> = (0..g.w).filter(|&k| k == 0 || (0..g.h).any(|r| g.at(r, k) != g.at(r, k - 1))).collect();
            map(g, rows.len(), cols.len(), |r, k| g.at(rows[r], cols[k]))
        }
        StripBgLines => {
            let bg = g.bg();
            let rows: Vec<usize> = (0..g.h).filter(|&r| (0..g.w).any(|k| g.at(r, k) != bg)).collect();
            let cols: Vec<usize> = (0..g.w).filter(|&k| (0..g.h).any(|r| g.at(r, k) != bg)).collect();
            if rows.is_empty() || cols.is_empty() { return None; }
            map(g, rows.len(), cols.len(), |r, k| g.at(rows[r], cols[k]))
        }
        FillEnclosed(c) => {
            let bg = g.bg(); if c == bg { return None; }
            let mut outside = vec![false; g.h * g.w];
            let mut stack: Vec<usize> = (0..g.h * g.w).filter(|&p| { let (r, k) = (p / g.w, p % g.w); (r == 0 || k == 0 || r + 1 == g.h || k + 1 == g.w) && g.c[p] == bg }).collect();
            for &p in &stack { outside[p] = true; }
            while let Some(p) = stack.pop() {
                let (r, k) = (p / g.w, p % g.w);
                let mut nb = Vec::with_capacity(4);
                if r > 0 { nb.push(p - g.w); } if r + 1 < g.h { nb.push(p + g.w); } if k > 0 { nb.push(p - 1); } if k + 1 < g.w { nb.push(p + 1); }
                for q in nb { if !outside[q] && g.c[q] == bg { outside[q] = true; stack.push(q); } }
            }
            if !(0..g.h * g.w).any(|p| g.c[p] == bg && !outside[p]) { return None; }
            let mut o = g.clone(); for p in 0..g.h * g.w { if g.c[p] == bg && !outside[p] { o.c[p] = c; } } o
        }
        OutlineObjects(c) => {
            let bg = g.bg(); if c == bg { return None; }
            let mut o = g.clone(); let mut any = false;
            for r in 0..g.h { for k in 0..g.w {
                if g.at(r, k) != bg { continue; }
                let mut near = false;
                for dr in -1i32..=1 { for dk in -1i32..=1 {
                    let (rr, kk) = (r as i32 + dr, k as i32 + dk);
                    if rr >= 0 && kk >= 0 && (rr as usize) < g.h && (kk as usize) < g.w && g.at(rr as usize, kk as usize) != bg && g.at(rr as usize, kk as usize) != c { near = true; }
                } }
                if near { o.set(r, k, c); any = true; }
            } }
            if !any { return None; } o
        }
        Sel(act, parse, pred) => select(g, act, parse, pred)?,
        EachObject(body) => each_object(g, body)?,
        Move(dir) => move_objects(g, dir)?,
        DownSample2 | DownSample3 => {
            let s = if op == DownSample2 { 2 } else { 3 };
            if g.h % s != 0 || g.w % s != 0 { return None; }
            map(g, g.h / s, g.w / s, |r, k| g.at(r * s, k * s))
        }
        Period => {
            let p = (1..=g.h).find(|&p| (0..g.h - p).all(|r| (0..g.w).all(|k| g.at(r, k) == g.at(r + p, k))))?;
            let q = (1..=g.w).find(|&q| (0..g.h).all(|r| (0..g.w - q).all(|k| g.at(r, k) == g.at(r, k + q))))?;
            if p == g.h && q == g.w { return None; }
            crop(g, 0, 0, p - 1, q - 1)?
        }
        UpByColours => { let bg = g.bg(); let n = (g.colours() & !(1u16 << bg)).count_ones() as usize; upscale(g, n)? }
        UpByCells => { let bg = g.bg(); let n = g.c.iter().filter(|&&v| v != bg).count(); upscale(g, n)? }
        FillInbox(c) => {
            let objs = objects(g); if objs.is_empty() { return None; }
            let mut o = g.clone(); let mut any = false;
            for cells in &objs {
                let (r0, k0, r1, k1) = bbox_of(cells);
                if r1 < r0 + 2 || k1 < k0 + 2 { continue; }
                for r in r0 + 1..r1 { for k in k0 + 1..k1 { if o.at(r, k) != c { o.set(r, k, c); any = true; } } }
            }
            if !any { return None; } o
        }
        BoxAround(a, c) => {
            let bg = g.bg(); if a == bg || c == bg || !has_colour(g, a) { return None; }
            let mut o = g.clone(); let mut any = false;
            for cells in objects_by(g, Parse::Mono).iter().filter(|o| g.at(o[0].0, o[0].1) == a) {
                let (r0, k0, r1, k1) = bbox_of(cells);
                let (a0, b0, a1, b1) = (r0 as i32 - 1, k0 as i32 - 1, r1 as i32 + 1, k1 as i32 + 1);
                for r in a0..=a1 { for k in b0..=b1 {
                    if r < 0 || k < 0 || r as usize >= g.h || k as usize >= g.w { continue; }
                    if (r == a0 || r == a1 || k == b0 || k == b1) && o.at(r as usize, k as usize) == bg { o.set(r as usize, k as usize, c); any = true; }
                } }
            }
            if !any { return None; } o
        }
        HaloOrth(a, c) => halo(g, a, c, &[(-1, 0), (1, 0), (0, -1), (0, 1)])?,
        HaloDiag(a, c) => halo(g, a, c, &[(-1, -1), (-1, 1), (1, -1), (1, 1)])?,
        LinesV(a, c) | LinesH(a, c) => {
            let bg = g.bg(); if a == bg || c == bg || !has_colour(g, a) { return None; }
            let v = matches!(op, LinesV(_, _));
            let mut o = g.clone(); let mut any = false;
            for r in 0..g.h { for k in 0..g.w {
                if g.at(r, k) != a { continue; }
                if v { for rr in 0..g.h { if o.at(rr, k) == bg { o.set(rr, k, c); any = true; } } }
                else { for kk in 0..g.w { if o.at(r, kk) == bg { o.set(r, kk, c); any = true; } } }
            } }
            if !any { return None; } o
        }
        ConnectWith(a, c) => {
            let bg = g.bg(); if a == bg || c == bg || !has_colour(g, a) { return None; }
            let mut o = g.clone(); let mut any = false;
            for r in 0..g.h {
                let ks: Vec<usize> = (0..g.w).filter(|&k| g.at(r, k) == a).collect();
                for w in ks.windows(2) { if (w[0] + 1..w[1]).all(|k| g.at(r, k) == bg) { for k in w[0] + 1..w[1] { o.set(r, k, c); any = true; } } }
            }
            for k in 0..g.w {
                let rs: Vec<usize> = (0..g.h).filter(|&r| g.at(r, k) == a).collect();
                for w in rs.windows(2) { if (w[0] + 1..w[1]).all(|r| g.at(r, k) == bg) { for r in w[0] + 1..w[1] { o.set(r, k, c); any = true; } } }
            }
            if !any { return None; } o
        }
        ShootOwn(dir) => {
            let bg = g.bg();
            let (dr, dk): (i32, i32) = match dir { Dir::Up => (-1, 0), Dir::Down => (1, 0), Dir::Left => (0, -1), Dir::Right => (0, 1) };
            let mut o = g.clone(); let mut any = false;
            for r in 0..g.h { for k in 0..g.w {
                let v = g.at(r, k); if v == bg { continue; }
                let (mut rr, mut kk) = (r as i32 + dr, k as i32 + dk);
                while rr >= 0 && kk >= 0 && (rr as usize) < g.h && (kk as usize) < g.w && g.at(rr as usize, kk as usize) == bg {
                    o.set(rr as usize, kk as usize, v); any = true; rr += dr; kk += dk;
                }
            } }
            if !any { return None; } o
        }
        CountRow => { let bg = g.bg(); let n = g.c.iter().filter(|&&v| v != bg).count(); if n == 0 || n > 30 { return None; } Grid::new(1, n, main_colour(g)?) }
        CountObjRow => { let n = objects(g).len(); if n == 0 || n > 30 { return None; } Grid::new(1, n, main_colour(g)?) }
        ColourOfLargest => {
            let objs = objects(g); let best = objs.iter().map(|o| o.len()).max()?;
            let hits: Vec<_> = objs.iter().filter(|o| o.len() == best).collect(); if hits.len() != 1 { return None; }
            let mut n = [0usize; 10]; for &(r, k) in hits[0] { n[g.at(r, k) as usize] += 1; }
            Grid::new(1, 1, (0..10).max_by_key(|&c| n[c]).unwrap() as u8)
        }
        Rot4Tile => {
            if g.h != g.w || g.h * 2 > 30 { return None; }
            let n = g.h;
            map(g, 2 * n, 2 * n, |r, k| { let (a, b) = (r % n, k % n); match (r / n, k / n) {
                (0, 0) => g.at(a, b), (0, 1) => g.at(n - 1 - b, a), (1, 1) => g.at(n - 1 - a, n - 1 - b), _ => g.at(b, n - 1 - a) } })
        }
        BoxObjects(c) | FillBoxes(c) => {
            let bg = g.bg(); if c == bg { return None; }
            let objs = objects(g); if objs.is_empty() { return None; }
            let mut o = g.clone(); let mut any = false;
            for cells in &objs {
                let r0 = cells.iter().map(|x| x.0).min()?; let r1 = cells.iter().map(|x| x.0).max()?;
                let k0 = cells.iter().map(|x| x.1).min()?; let k1 = cells.iter().map(|x| x.1).max()?;
                if let BoxObjects(_) = op {
                    let (a0, b0, a1, b1) = (r0 as i32 - 1, k0 as i32 - 1, r1 as i32 + 1, k1 as i32 + 1);
                    for r in a0..=a1 { for k in b0..=b1 {
                        if r < 0 || k < 0 || r as usize >= g.h || k as usize >= g.w { continue; }
                        if (r == a0 || r == a1 || k == b0 || k == b1) && o.at(r as usize, k as usize) == bg { o.set(r as usize, k as usize, c); any = true; }
                    } }
                } else {
                    for r in r0..=r1 { for k in k0..=k1 { if o.at(r, k) == bg { o.set(r, k, c); any = true; } } }
                }
            }
            if !any { return None; } o
        }
        ShootUp(c) => shoot(g, c, -1, 0)?, ShootDown(c) => shoot(g, c, 1, 0)?,
        ShootLeft(c) => shoot(g, c, 0, -1)?, ShootRight(c) => shoot(g, c, 0, 1)?,
        Connect(c) => {
            let bg = g.bg(); if c == bg { return None; }
            let mut o = g.clone(); let mut any = false;
            for r in 0..g.h {
                let ks: Vec<usize> = (0..g.w).filter(|&k| g.at(r, k) == c).collect();
                for w in ks.windows(2) { for k in w[0] + 1..w[1] { if o.at(r, k) == bg { o.set(r, k, c); any = true; } } }
            }
            for k in 0..g.w {
                let rs: Vec<usize> = (0..g.h).filter(|&r| g.at(r, k) == c).collect();
                for w in rs.windows(2) { for r in w[0] + 1..w[1] { if o.at(r, k) == bg { o.set(r, k, c); any = true; } } }
            }
            if !any { return None; } o
        }
        SymH => symmetrize(g, true, false, false)?, SymV => symmetrize(g, false, true, false)?,
        Sym4 => symmetrize(g, true, true, false)?, SymD => symmetrize(g, false, false, true)?,
        CropInside(c) => {
            let (r0, k0, r1, k1) = bbox(g, |v| v == c)?;
            if r1 < r0 + 2 || k1 < k0 + 2 { return None; }
            crop(g, r0 + 1, k0 + 1, r1 - 1, k1 - 1)?
        }
        SelfKron => {
            if g.h * g.h > 30 || g.w * g.w > 30 { return None; }
            let bg = g.bg();
            map(g, g.h * g.h, g.w * g.w, |r, k| if g.at(r / g.h, k / g.w) != bg { g.at(r % g.h, k % g.w) } else { bg })
        }
    };
    if out.valid() { Some(out) } else { None }
}
