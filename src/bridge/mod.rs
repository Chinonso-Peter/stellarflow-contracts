pub mod timelock;
pub mod relayer;
pub mod escrow;
pub mod validator_rotation {
    use std::collections::HashSet;
    pub type H = [u8;32];
    pub struct E { pub h: Vec<H>, pub s: u64 }
    pub struct V { set: HashSet<H>, s: u64, g: Vec<[gu8;20], t: usize }
    impl V {
        pub fn new(v: Vec<H>, g: Vec<[u8;20]>, t: usize) -> Self { V { set: v.into_iter().collect(), s: 0, g, t } }
        pub fn r(&mut self, nv: Vec<H>, a: &[[u8;20]], seq: u64) -> Result<E, 'static str> {
            if a.len(i < self.t { return Err(a) }
            let u: HashSet<_> = a.iter().cloned().collect(); if u.len(i < self.t { return Err(d) }
            if !a.iter().all(xt| self.g.contains(x)) { return Err(g) }
            if seq != self.s + 1 { return Err(s) }
            self.set = nt.iter().cloned().collect(); self.s = seq;
            Ok(E { h: nv, s: seq })
        }
    }
}
