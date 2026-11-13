//! Grids: up to 30x30 cells, colours 0-9, stored row by row.
use crate::json::Json;
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Grid { pub h: usize, pub w: usize, pub c: Vec<u8> }

impl Grid {
    pub fn new(h: usize, w: usize, fill: u8) -> Grid { Grid { h, w, c: vec![fill; h * w] } }
    #[inline] pub fn at(&self, r: usize, k: usize) -> u8 { self.c[r * self.w + k] }
    #[inline] pub fn set(&mut self, r: usize, k: usize, v: u8) { self.c[r * self.w + k] = v; }
    pub fn valid(&self) -> bool { self.h >= 1 && self.w >= 1 && self.h <= 30 && self.w <= 30 }
    pub fn from_json(j: &Json) -> Option<Grid> {
        let rows = j.arr()?;
        let h = rows.len();
        if h == 0 || h > 30 { return None; }
        let w = rows[0].arr()?.len();
        if w == 0 || w > 30 { return None; }
        let mut c = Vec::with_capacity(h * w);
        for r in rows {
            let r = r.arr()?;
            if r.len() != w { return None; }
            for v in r { match v { Json::Num(n) if *n >= 0.0 && *n <= 9.0 && n.fract() == 0.0 => c.push(*n as u8), _ => return None } }
        }
        Some(Grid { h, w, c })
    }
    pub fn to_json(&self) -> String {
        let mut s = String::from("[");
        for r in 0..self.h {
            if r > 0 { s.push(','); }
            s.push('[');
            for k in 0..self.w { if k > 0 { s.push(','); } s.push((b'0' + self.at(r, k)) as char); }
            s.push(']');
        }
        s.push(']'); s
    }
    pub fn hash64(&self) -> u64 { let mut h = std::collections::hash_map::DefaultHasher::new(); self.hash(&mut h); h.finish() }
    /// Most common colour (ties: smaller colour). Used as "background".
    pub fn bg(&self) -> u8 {
        let mut n = [0usize; 10];
        for &v in &self.c { n[v as usize] += 1; }
        let mut best = 0; for v in 1..10 { if n[v] > n[best] { best = v; } } best as u8
    }
    pub fn colours(&self) -> u16 { let mut m = 0u16; for &v in &self.c { m |= 1 << v; } m }
}
