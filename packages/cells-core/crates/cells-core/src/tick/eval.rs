//! Recompute strategies. All take the changed essential cells as seeds and
//! append the derived cells whose value changed. They must agree on results;
//! they differ only in how much work they do.

use crate::document::CellIdx;
use crate::program::Program;

pub trait Evaluator {
    fn name(&self) -> &'static str;
    /// `changed` holds the essential cells written this tick on entry.
    fn recompute(&mut self, program: &Program, cells: &mut [f64], changed: &mut Vec<CellIdx>);
}

/// Run every instruction in schedule order, ignoring the seeds.
#[derive(Default)]
pub struct FullRecompute;

impl Evaluator for FullRecompute {
    fn name(&self) -> &'static str {
        "full"
    }
    fn recompute(&mut self, program: &Program, cells: &mut [f64], changed: &mut Vec<CellIdx>) {
        program.run_all_tracking(cells, changed);
    }
}

/// Walk only the downstream closure of the seeds, in schedule order. The
/// schedule is topological, so every instruction a step queues lies after
/// it: a bitset over schedule positions, scanned forward a word at a time,
/// replaces a priority queue. Work is proportional to the closure plus its
/// span over 64.
///
/// Readers are stored as word masks keyed by the schedule position of the
/// instruction that writes, so finding the next instruction to run never
/// waits on reading back which cell changed: a step's readers in its own
/// word are one mask, those in later words a short list of masks.
#[derive(Debug, Clone)]
pub struct DirtyClosure {
    /// readers[i] = where instruction `i`'s readers are, one record per
    /// instruction so a step touches one line of it.
    readers: Vec<Readers>,
    far: Vec<Marks>,
    /// cell_marks[cell_start[cell]..cell_start[cell + 1]] = readers of
    /// `cell`, one mask per word, ascending: for the seeds and the outputs
    /// of instructions with several outputs; empty for every other cell.
    cell_start: Vec<u32>,
    cell_marks: Vec<Marks>,
    /// One bit per schedule position; all clear between ticks.
    dirty: Vec<u64>,
}

/// The readers of one instruction's output.
#[derive(Debug, Clone, Copy, Default)]
struct Readers {
    /// Readers within the instruction's own word.
    near: u64,
    /// far[far_start..far_start + far_len] = readers in later words, one
    /// mask per word, ascending. `far_len` is `MULTI` for an instruction
    /// with several outputs, whose readers are marked per changed output
    /// through `cell_marks`, so an output that did not change marks nothing.
    far_start: u32,
    far_len: u32,
}

const MULTI: u32 = u32::MAX;

/// Instructions to mark in one word of the bitset.
#[derive(Debug, Clone, Copy)]
struct Marks {
    word: u32,
    mask: u64,
}

/// Append the ascending schedule positions `readers`, one mask per word.
fn group(readers: &[u32], out: &mut Vec<Marks>) {
    let start = out.len();
    for &i in readers {
        let (word, bit) = (i / 64, 1u64 << (i % 64));
        match out[start..].last_mut() {
            Some(m) if m.word == word => m.mask |= bit,
            _ => out.push(Marks { word, mask: bit }),
        }
    }
}

impl DirtyClosure {
    pub fn new(program: &Program, n_cells: usize) -> Self {
        // The readers of each cell, ascending, in compressed-sparse-row form.
        let mut dep_start = vec![0u32; n_cells + 1];
        for ins in &program.instrs {
            for input in ins.op.inputs(&program.operands) {
                dep_start[input as usize + 1] += 1;
            }
        }
        for c in 0..n_cells {
            dep_start[c + 1] += dep_start[c];
        }
        let mut fill = dep_start.clone();
        let mut dependents = vec![0u32; dep_start[n_cells] as usize];
        for (i, ins) in program.instrs.iter().enumerate() {
            for input in ins.op.inputs(&program.operands) {
                dependents[fill[input as usize] as usize] = i as u32;
                fill[input as usize] += 1;
            }
        }
        let readers_of = |c: usize| &dependents[dep_start[c] as usize..dep_start[c + 1] as usize];

        // Only cells no instruction writes (the seeds) and the outputs of
        // instructions with several outputs are marked per cell; every
        // other cell's readers are in its writer's record.
        let per_cell = |c: usize| {
            let p = program.producer[c];
            p == u32::MAX || program.instrs[p as usize].op.n_out() != 1
        };
        let mut cell_start = Vec::with_capacity(n_cells + 1);
        let mut cell_marks = Vec::new();
        cell_start.push(0);
        for c in 0..n_cells {
            if per_cell(c) {
                group(readers_of(c), &mut cell_marks);
            }
            cell_start.push(cell_marks.len() as u32);
        }

        let n = program.len();
        let mut readers = vec![Readers::default(); n];
        let mut far = Vec::new();
        let mut marks = Vec::new();
        for (i, ins) in program.instrs.iter().enumerate() {
            let r = &mut readers[i];
            r.far_start = far.len() as u32;
            if ins.op.n_out() == 1 {
                marks.clear();
                group(readers_of(ins.out as usize), &mut marks);
                for m in &marks {
                    if m.word as usize == i / 64 {
                        r.near = m.mask;
                    } else {
                        far.push(*m);
                    }
                }
                r.far_len = far.len() as u32 - r.far_start;
            } else {
                r.far_len = MULTI;
            }
        }
        DirtyClosure {
            readers,
            far,
            cell_start,
            cell_marks,
            dirty: vec![0; n.div_ceil(64)],
        }
    }
}

/// How a document recomputes after a tick's writes: what it holds and
/// ticks with (`Document::request`). A dirty closure's tables follow the
/// program they were built for: they are built on the first tick, and a
/// rebuild's document starts without them (`fresh`), so no caller keeps
/// them in step by hand and a document never ticked never holds them.
#[derive(Debug, Clone)]
pub enum Recompute {
    /// Every instruction (`FullRecompute`).
    Full,
    /// The closure of what changed (`DirtyClosure`), what the browser runs;
    /// None until the first tick.
    DirtyClosure(Option<DirtyClosure>),
}

impl Default for Recompute {
    fn default() -> Self {
        Recompute::DirtyClosure(None)
    }
}

impl Recompute {
    /// The strategy called `name`: "full" or "dirty-closure".
    pub fn named(name: &str) -> Option<Recompute> {
        match name {
            "full" => Some(Recompute::Full),
            "dirty-closure" => Some(Recompute::DirtyClosure(None)),
            _ => None,
        }
    }

    /// The same strategy, for another program: without tables.
    pub fn fresh(&self) -> Recompute {
        match self {
            Recompute::Full => Recompute::Full,
            Recompute::DirtyClosure(_) => Recompute::DirtyClosure(None),
        }
    }
}

impl Evaluator for Recompute {
    fn name(&self) -> &'static str {
        match self {
            Recompute::Full => "full",
            Recompute::DirtyClosure(_) => "dirty-closure",
        }
    }
    fn recompute(&mut self, program: &Program, cells: &mut [f64], changed: &mut Vec<CellIdx>) {
        match self {
            Recompute::Full => FullRecompute.recompute(program, cells, changed),
            Recompute::DirtyClosure(d) => d
                .get_or_insert_with(|| DirtyClosure::new(program, cells.len()))
                .recompute(program, cells, changed),
        }
    }
}

impl Evaluator for DirtyClosure {
    fn name(&self) -> &'static str {
        "dirty-closure"
    }
    fn recompute(&mut self, program: &Program, cells: &mut [f64], changed: &mut Vec<CellIdx>) {
        let mut lo = usize::MAX;
        let mut hi = 0;
        for &c in changed.iter() {
            let (s, e) = (
                self.cell_start[c as usize] as usize,
                self.cell_start[c as usize + 1] as usize,
            );
            for m in &self.cell_marks[s..e] {
                self.dirty[m.word as usize] |= m.mask;
            }
            if s < e {
                lo = lo.min(self.cell_marks[s].word as usize);
                hi = hi.max(self.cell_marks[e - 1].word as usize);
            }
        }
        if lo == usize::MAX {
            return;
        }
        let mut w = lo;
        while w <= hi {
            let mut bits = std::mem::take(&mut self.dirty[w]);
            while bits != 0 {
                let b = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                let i = w * 64 + b;
                let before = changed.len();
                program.step(&program.instrs[i], cells, Some(changed));
                if changed.len() == before {
                    continue;
                }
                // Readers come later in the schedule: above bit `b`, or in a later word.
                let r = self.readers[i];
                if r.far_len != MULTI {
                    bits |= r.near;
                    let (s, e) = (r.far_start as usize, (r.far_start + r.far_len) as usize);
                    for m in &self.far[s..e] {
                        self.dirty[m.word as usize] |= m.mask;
                    }
                    if s < e {
                        hi = hi.max(self.far[e - 1].word as usize);
                    }
                } else {
                    for &c in &changed[before..] {
                        let (s, e) = (
                            self.cell_start[c as usize] as usize,
                            self.cell_start[c as usize + 1] as usize,
                        );
                        for m in &self.cell_marks[s..e] {
                            if m.word as usize == w {
                                bits |= m.mask;
                            } else {
                                self.dirty[m.word as usize] |= m.mask;
                            }
                        }
                        if s < e {
                            hi = hi.max(self.cell_marks[e - 1].word as usize);
                        }
                    }
                }
            }
            w += 1;
        }
    }
}
