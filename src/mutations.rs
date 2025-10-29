//! Mux the mutators based on weighted scores.
//!
//! ## MUTATIONS:
//!
//! > **DEFAULT:** `ft=2,fo=2,fn,num=5,ld,lds,lr2,li,ls,lp,lr,sr,sd,bd,bf,bi,br,bp,bei,bed,ber,uw,ui=2`
//!
//! | id |complete | desc | func |
//! |----|---------|------|------|
//! |`ab`|&check;|enhance silly issues in ASCII string data handling||
//! |`bd`|&check;|drop a byte|[sed_byte_drop]|
//! |`bed`|&check;|decrement a byte by one|[sed_byte_dec]
//! |`bei`|&check;|increment a byte by one|[sed_byte_inc]
//! |`ber`|&check;|swap a byte with a random one|[sed_byte_random]
//! |`bf`|&check;| flip one bit|[sed_byte_flip]
//! |`bi`|&check;| insert a random byte|[sed_byte_insert]
//! |`bp`|&check;| permute some bytes|[sed_byte_perm]
//! |`br`|&check;| repeat a byte|[sed_byte_repeat]
//! |`fn`|&check;| likely clone data between similar positions|
//! |`fo`|&check;| fuse previously seen data elsewhere|
//! |`ft`|&check;| jump to a similar position in block|
//! |`ld`|&check;| delete a line|[sed_line_del]
//! |`lds`|&check;|delete many lines|[sed_line_del_seq]
//! |`li`|&check;| copy a line closeby|[sed_line_clone]
//! |`lis`|&check;|insert a line from elsewhere|[sed_line_ins]
//! |`lp`|&check;| swap order of lines|[sed_line_perm]
//! |`lr`|&check;| repeat a line|[sed_line_repeat]
//! |`lr2`|&check;|duplicate a line|[sed_line_dup]
//! |`lrs`|&check;|replace a line with one from elsewhere|[sed_line_replace]
//! |`ls`|&check;| swap two lines|[sed_line_swap]
//! |`nop`|&check;|do nothing (debug/test)|[nop]
//! |`num`|&check;|try to modify a textual number|[sed_num]
//! |`sd`|&check;| delete a sequence of bytes|[sed_seq_del]
//! |`sr`|&check;| repeat a sequence of bytes|[sed_seq_repeat]
//! |str|&cross;|try to modify a string|
//! |`td`|&check;| delete a node|[sed_tree_del]
//! |`tr`|&check;| repeat a path of the parse tree|[sed_tree_stutter]
//! |`tr2`|&check;|duplicate a node|[sed_tree_dup]
//! |`ts1`|&check;|swap one node with another one|[sed_tree_swap1]
//! |`ts2`|&check;|swap two nodes pairwise|[sed_tree_swap2]
//! |`ui`|&check;| insert funny unicode|[sed_utf8_insert]
//! |`uw`|&check;| try to make a code point too wide|[sed_utf8_widen]
//! |word|&cross;|   try to play with what look like n-byte words or values|
//! |xp|&cross;| try to parse XML and mutate it|

// TODO: byte inversion
// Even powers of two, +/- a random value from 0..16
// Add/subtract a random value from 0..16
// Overwrite contents with zero bytes

use std::collections::BTreeMap;
#[cfg(test)]
use std::println as debug;

use ethnum::*;
#[cfg(not(test))]
use log::debug;
use rand::RngCore;
use rand::{seq::SliceRandom, Rng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::shared::*;

// pub const DEFAULT_MUTATIONS: &'static str = "ft=2,fo=2,fn,num=5,td,tr2,ts1,tr,ts2,ld,lds,lr2,li,ls,lp,lr,lis,lrs,sr,sd,bd,bf,bi,br,bp,bei,bed,ber,uw,ui=2,xp=9,ab";
pub const DEFAULT_MUTATIONS: &str =
    "ft=2,fo=2,fn,num=5,ld,lds,lr2,li,ls,lp,lr,sr,sd,bd,bf,bi,br,bp,bei,bed,ber,uw,ui=2,ab";
const MAX_SCORE: usize = 10;
const MIN_SCORE: usize = 2;

macro_rules! muta {
    ($($x:expr),*) => (Mutator::new($($x),*));
}

/// Mutator
#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum MutaType {
    AsciiBad,
    ByteDrop,
    ByteFlip,
    ByteInsert,
    ByteRepeat,
    BytePerm,
    ByteInc,
    ByteDec,
    ByteRand,
    SeqRepeat,
    SeqDel,
    LineDel,
    LineDelSeq,
    LineDup,
    LineClone,
    LineRepeat,
    LineSwap,
    LinePerm,
    LineIns,
    LineReplace,
    TreeDel,
    TreeDup,
    TreeSwap1,
    TreeSwap2,
    TreeRepeat,
    UTF8Widen,
    UTF8Insert,
    Num,
    Str,
    Word,
    Xp,
    FuseThis,
    FuseNext,
    FuseOld,
    Nop,
}

#[allow(clippy::enum_glob_use)]
impl MutaType {
    #[must_use]
    pub fn id(&self) -> String {
        use MutaType::*;
        let id = match *self {
            AsciiBad => "ab",
            ByteDrop => "bd",
            ByteFlip => "bf",
            ByteInsert => "bi",
            ByteRepeat => "br",
            BytePerm => "bp",
            ByteInc => "bei",
            ByteDec => "bed",
            ByteRand => "ber",
            SeqRepeat => "sr",
            SeqDel => "sd",
            LineDel => "ld",
            LineDelSeq => "lds",
            LineDup => "lr2",
            LineClone => "li",
            LineRepeat => "lr",
            LineSwap => "ls",
            LinePerm => "lp",
            LineIns => "lis",
            LineReplace => "lrs",
            TreeDel => "td",
            TreeDup => "tr2",
            TreeSwap1 => "ts1",
            TreeSwap2 => "ts2",
            TreeRepeat => "tr",
            UTF8Widen => "uw",
            UTF8Insert => "ui",
            Num => "num",
            Str => "str",
            Word => "word",
            Xp => "xp",
            FuseThis => "ft",
            FuseNext => "fn",
            FuseOld => "fo",
            Nop => "nop",
        };
        id.to_string()
    }

    #[must_use]
    #[allow(clippy::enum_glob_use)]
    pub fn info(&self) -> String {
        use MutaType::*;
        let info = match *self {
            AsciiBad => "enhance silly issues in ASCII string data handling",
            ByteDrop => "drop a byte",
            ByteFlip => "flip one bit",
            ByteInsert => "insert a random byte",
            ByteRepeat => "repeat a byte",
            BytePerm => "permute some bytes",
            ByteInc => "increment a byte by one",
            ByteDec => "decrement a byte by one",
            ByteRand => "swap a byte with a random one",
            SeqRepeat => "repeat a sequence of bytes",
            SeqDel => "delete a sequence of bytes",
            LineDel => "delete a line",
            LineDelSeq => "delete many lines",
            LineDup => "duplicate a line",
            LineClone => "copy a line closeby",
            LineRepeat => "repeat a line",
            LineSwap => "swap two lines",
            LinePerm => "swap order of lines",
            LineIns => "insert a line from elsewhere",
            LineReplace => "replace a line with one from elsewhere",
            TreeDel => "delete a node",
            TreeDup => "duplicate a node",
            TreeSwap1 => "swap one node with another one",
            TreeSwap2 => "swap two nodes pairwise",
            TreeRepeat => "repeat a path of the parse tree",
            UTF8Widen => "try to make a code point too wide",
            UTF8Insert => "insert funny unicode",
            Num => "try to modify a textual number",
            Str => "try to modify a string",
            Word => "try to play with what look like n-byte words or values",
            Xp => "try to parse XML and mutate it",
            FuseThis => "jump to a similar position in block",
            FuseNext => "likely clone data between similar positions",
            FuseOld => "fuse previously seen data elsewhere",
            Nop => "do nothing (debug/test)",
        };
        info.to_string()
    }

    #[allow(clippy::enum_glob_use)]
    fn mutate(self, rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
        use MutaType::*;
        match self {
            AsciiBad => ascii_bad(rng, data),
            ByteDrop => sed_byte_drop(rng, data),
            ByteFlip => sed_byte_flip(rng, data),
            ByteInsert => sed_byte_insert(rng, data),
            ByteRepeat => sed_byte_repeat(rng, data),
            BytePerm => sed_byte_perm(rng, data),
            ByteInc => sed_byte_inc(rng, data),
            ByteDec => sed_byte_dec(rng, data),
            ByteRand => sed_byte_random(rng, data),
            SeqRepeat => sed_seq_repeat(rng, data),
            SeqDel => sed_seq_del(rng, data),
            LineDel => sed_line_del(rng, data),
            LineDelSeq => sed_line_del_seq(rng, data),
            LineDup => sed_line_dup(rng, data),
            LineClone => sed_line_clone(rng, data),
            LineRepeat => sed_line_repeat(rng, data),
            LineSwap => sed_line_swap(rng, data),
            LinePerm => sed_line_perm(rng, data),
            LineIns => sed_line_ins(rng, data),
            LineReplace => sed_line_replace(rng, data),
            TreeDel => sed_tree_del(rng, data),
            TreeDup => sed_tree_dup(rng, data),
            TreeSwap1 => sed_tree_swap1(rng, data),
            TreeSwap2 => sed_tree_swap2(rng, data),
            TreeRepeat => sed_tree_stutter(rng, data),
            UTF8Widen => sed_utf8_widen(rng, data),
            UTF8Insert => sed_utf8_insert(rng, data),
            Num => sed_num(rng, data),
            Str | Word | Xp => (None, 0),
            FuseThis => sed_fuse_this(rng, data),
            FuseNext => sed_fuse_next(rng, data),
            FuseOld => sed_fuse_old(rng, data),
            Nop => nop(rng, data),
        }
    }

    #[must_use]
    pub fn id_to_mutatype(id: &str) -> Option<Self> {
        Self::iter().find(|&muta| muta.id() == id)
    }
}

#[must_use]
pub fn init_mutations() -> BTreeMap<MutaType, Mutator> {
    let mut map = BTreeMap::<MutaType, Mutator>::new();
    for muta in MutaType::iter() {
        map.insert(muta, muta!(muta));
    }
    map
}

// rs ll delta
pub type MutatorFunc =
    fn(rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize);

#[derive(Debug)]
pub struct Mutations {
    pub mutators: BTreeMap<MutaType, Mutator>,
    pub mutator_nodes: Vec<MutaType>,
    pub mutas: Option<Vec<MutaType>>,
}

pub struct Mutator {
    pub muta: MutaType,
    /// Activation probability is (score*priority)/SUM(total-scores)
    pub priority: usize,
    pub score: usize,
    pub weight: usize,
    pub delta: isize,
}

impl std::fmt::Debug for Mutator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Mutator")
            .field("id", &self.id())
            .field("info", &self.info())
            .field("priority", &self.priority)
            .field("score", &self.score)
            .field("weight", &self.weight)
            .field("delta", &self.delta)
            .finish_non_exhaustive()
    }
}

impl Mutator {
    #[must_use]
    pub const fn new(muta: MutaType) -> Self {
        Self {
            muta,
            priority: 0,
            score: MAX_SCORE,
            weight: 0,
            delta: 0,
        }
    }

    #[must_use]
    pub fn id(&self) -> String {
        self.muta.id()
    }

    #[must_use]
    pub fn info(&self) -> String {
        self.muta.info()
    }
}

#[allow(clippy::new_without_default)]
impl Mutations {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mutators: BTreeMap::new(),
            mutator_nodes: Vec::new(),
            mutas: None,
        }
    }

    pub fn init(&mut self) {
        self.mutators = init_mutations();
    }

    pub fn default_mutations(&mut self) {
        self.mutator_nodes = string_mutators(DEFAULT_MUTATIONS, &mut self.mutators);
    }

    // Activation probability is (score*priority)/SUM(total-scores)
    pub fn randomize(&mut self, rng: &mut dyn RngCore) {
        if self.mutas.is_some() {
            // Apply random scores
            for mutator in self.mutators.values_mut() {
                if self.mutator_nodes.contains(&mutator.muta) {
                    let rand_value = MAX_SCORE.rands(rng);
                    mutator.score = if rand_value < 2 {
                        MIN_SCORE
                    } else {
                        rand_value
                    };
                }
            }
        }
        self.mutator_nodes
            .retain(|r| self.mutators.iter().any(|m| m.0 == r));
        self.mutas = Some(self.mutator_nodes.clone());
    }

    fn weighted_permutation(&mut self, rng: &mut dyn RngCore) -> Vec<&mut Mutator> {
        let mut out_mutas: Vec<&mut Mutator> = vec![];
        for m in self.mutators.values_mut() {
            if let Some(mutas) = &self.mutas {
                if mutas.contains(&m.muta) && m.priority > 0 {
                    m.weight = (m.priority * m.score).rands(rng);
                    out_mutas.push(m);
                }
            } else if self.mutator_nodes.contains(&m.muta) && m.priority > 0 {
                m.weight = (m.priority * m.score).rands(rng);
                out_mutas.push(m);
            }
        }
        // Sort by weight
        out_mutas.sort_by(|x, y| x.weight.cmp(&y.weight));
        // save mutas
        self.mutas = Some(out_mutas.iter().map(|x| x.muta).collect());

        out_mutas
    }

    pub fn mux_fuzzers(
        &mut self,
        rng: &mut dyn RngCore,
        maybe_data: Option<&Vec<u8>>,
    ) -> Option<Vec<u8>> {
        let mut mutas = self.weighted_permutation(rng);
        let data = maybe_data?;
        while !mutas.is_empty() {
            let muta = mutas.pop()?;
            debug!("muta {}", muta.id());
            match muta.muta.mutate(rng, Some(data)) {
                (Some(new_data), delta) => {
                    // always remember whatever was learned
                    muta.score = adjust_priority(muta.score, delta);
                    muta.delta = delta;
                    if new_data == *data {
                        debug!("Nothing changed");
                    } else {
                        return Some(new_data);
                    }
                }
                _ => {
                    debug!("Nothing changed");
                }
            }
        }
        maybe_data.cloned()
    }
}

/// This function parses mutation string i.e. ft=2,fo=2
pub fn string_mutators(input: &str, mutators: &mut BTreeMap<MutaType, Mutator>) -> Vec<MutaType> {
    let mut applied_mutators: Vec<MutaType> = vec![];
    let string_list = input.trim().split(',').collect::<Vec<&str>>();
    //debug!("mutators {:#?}", _mutators);
    for s in string_list {
        let tuple = s.trim().split('=').collect::<Vec<&str>>();
        let mutator_id = tuple.first().unwrap_or(&"").trim();
        let priority = tuple
            .get(1)
            .unwrap_or(&"0")
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        let muta_type = MutaType::id_to_mutatype(mutator_id).unwrap_or(MutaType::Nop);
        if let Some(mutator) = mutators.get_mut(&muta_type) {
            mutator.priority = if priority < 1 { 1 } else { priority };
            mutator.score = MAX_SCORE;
            applied_mutators.push(mutator.muta);
        } else {
            panic!("unknown mutator {mutator_id}");
        }
    }
    applied_mutators
}

fn rand_delta(rng: &mut dyn RngCore) -> isize {
    if rng.r#gen() {
        1
    } else {
        -1
    }
}

fn rand_delta_up(rng: &mut dyn RngCore) -> isize {
    // slight positive bias
    if rng.gen_range(0..20) <= 11 {
        1
    } else {
        -1
    }
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
fn adjust_priority(pri: usize, delta: isize) -> usize {
    match delta {
        0 => pri,
        _ => std::cmp::max(
            MIN_SCORE as isize,
            std::cmp::max(MAX_SCORE as isize, pri as isize + delta),
        ) as usize,
    }
}

// Number Mutators

// get digit
#[allow(clippy::cast_possible_truncation)]
fn get_num(data: Option<&[u8]>) -> (Option<i256>, Option<usize>) {
    let mut out = vec![];
    let mut n = i256::from(0);
    if let Some(data) = data {
        for val in data {
            if char::from(*val).is_ascii_digit() {
                out.push(*val);
            } else {
                break;
            }
        }
        if out.is_empty() {
            if data.is_empty() {
                return (None, None);
            }
            return (None, Some(1));
        }
        for (pos, m) in out.iter().rev().enumerate() {
            let num = i256::from(char::from(*m).to_digit(10).unwrap_or(0));
            if pos == 0 {
                n = num;
            } else {
                n += num
                    .overflowing_mul(i256::from(10_usize.overflowing_pow(pos as u32).0 as i128))
                    .0;
            }
        }
        return (Some(n), Some(out.len()));
    }
    (None, None)
}

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
fn mutate_a_num(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (isize, Option<Vec<u8>>) {
    let mut offset = 0_usize;
    let mut nfound = 0_usize;
    if let Some(data) = data {
        if data.is_empty() {
            return (0, None);
        }
        let mut num_offsets: Vec<(i256, usize, usize)> = vec![];
        while offset < data.len() {
            if let (Some(val), Some(len)) = get_num(Some(&data[offset..])) {
                nfound += 1;
                num_offsets.push((val, offset, len));
                offset += len;
                continue;
            }
            offset += 1;
        }
        let which = match nfound {
            0 => return (0, None),
            _ => nfound.rands(rng) as isize,
        };
        if let Some(target) = num_offsets.get(which as usize) {
            let new_num = mutate_num(rng, target.0).to_string().as_bytes().to_vec();
            let mut new_lst = data[..target.1].to_vec();
            new_lst.extend(new_num);
            new_lst.extend(&data[target.1 + target.2..]);
            return (which + 1, Some(new_lst));
        }
    }
    (0, None)
}

// -> lst, delta
pub fn sed_num(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let (which, data) = mutate_a_num(rng, data);
    // check if binary
    let is_bin = data.as_ref().is_some_and(|d| is_binarish(Some(d)));
    if which == 0 {
        //textual with less frequent numbers
        debug!("textual with less numbers");
        let n = 10_usize.rands(rng);
        if n == 0 {
            return (data, -1);
        }
        return (data, 0);
    } else if is_bin {
        return (data, -1);
    }
    (data, 2)
}

// Byte-level Mutations

pub fn sed_byte_drop(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    if !data.is_empty() {
        let p = rng.gen_range(0..data.len());
        new_data.remove(p);
    }
    (Some(new_data), d)
}

pub fn sed_byte_inc(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    if !data.is_empty() {
        let p = rng.gen_range(0..data.len());
        new_data[p] = new_data[p].wrapping_add(1);
    }
    (Some(new_data), d)
}

pub fn sed_byte_dec(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    if !data.is_empty() {
        let p = rng.gen_range(0..data.len());
        new_data[p] = new_data[p].wrapping_sub(1);
    }
    (Some(new_data), d)
}

pub fn sed_byte_flip(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    if !data.is_empty() {
        let b = 1 << rng.gen_range(0..8);
        let p = rng.gen_range(0..data.len());
        new_data[p] ^= b;
    }
    (Some(new_data), d)
}

pub fn sed_byte_insert(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    let b = rng.r#gen::<u8>();
    let p = rng.gen_range(0..=data.len());
    new_data.insert(p, b);
    (Some(new_data), d)
}

fn repeat_len(rng: &mut dyn RngCore) -> usize {
    let mut limit = 0b10;
    while rng.r#gen() && limit != 0x20000 {
        limit <<= 1;
    }
    rng.gen_range(0..limit)
}

pub fn sed_byte_repeat(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let n = repeat_len(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    if !data.is_empty() {
        let p = rng.gen_range(0..data.len());
        let to_repeat = data[p];
        for _ in 0..n {
            // repeat data[p] n times
            new_data.insert(p, to_repeat);
        }
    }
    (Some(new_data), d)
}

pub fn sed_byte_random(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    if !data.is_empty() {
        let b = rng.r#gen::<u8>();
        let p = rng.gen_range(0..data.len());
        new_data[p] = b;
    }
    (Some(new_data), d)
}

pub fn sed_byte_perm(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    if !data.is_empty() {
        let p = rng.gen_range(0..data.len());
        let n = std::cmp::min(p + rng.gen_range(2..20), new_data.len());
        new_data[p..n].shuffle(rng);
    }
    (Some(new_data), d)
}

pub fn sed_utf8_widen(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    if !data.is_empty() {
        let p = rng.gen_range(0..data.len());
        // assuming we hit a 6-bit ascii char, make it unnecessarily wide
        // which might confuse a length calculation
        let b = new_data[p];
        if b == (b & 0b0011_1111) {
            new_data[p] = 0b1100_0000;
            new_data.insert(p + 1, b | 0b1000_0000);
        }
    }
    (Some(new_data), d)
}

static FUNNY_UNICODE: std::sync::LazyLock<Vec<Vec<u8>>> = std::sync::LazyLock::new(|| {
    enum Range {
        Interval(u32, u32),
        Scalar(u32),
    }

    let mut ret = Vec::new();
    ret.push("\u{202E}".to_string().into_bytes()); // Right to Left Override
    ret.push("\u{202D}".to_string().into_bytes()); // Left to Right Override
    ret.push("\u{180E}".to_string().into_bytes()); // Mongolian Vowel Separator
    ret.push("\u{2060}".to_string().into_bytes()); // Word Joiner
    ret.push("\u{FEFE}".to_string().into_bytes()); // reserved
    ret.push("\u{FFFF}".to_string().into_bytes()); // not a character
    ret.push("\u{0FED}".to_string().into_bytes()); // unassigned
    ret.push(vec![0xed, 0xba, 0xad]); // U+DEAD illegal low surrogate
    ret.push(vec![0xed, 0xaa, 0xad]); // U+DAAD illegal high surrogate
    ret.push("\u{F8FF}".to_string().into_bytes()); // private use char (Apple)
    ret.push("\u{FF0F}".to_string().into_bytes()); // full width solidus
    ret.push("\u{1D7D6}".to_string().into_bytes()); // MATHEMATICAL BOLD DIGIT EIGHT
    ret.push("\u{00DF}".to_string().into_bytes()); // IDNA deviant
    ret.push("\u{FDFD}".to_string().into_bytes()); // expands by 11x (UTF-8) and 18x (UTF-16) NFKC
    ret.push("\u{0390}".to_string().into_bytes()); // expands by 3x (UTF-8) NFD
    ret.push("\u{1F82}".to_string().into_bytes()); // expands by 4x (UTF-16) NFD
    ret.push("\u{FB2C}".to_string().into_bytes()); // expands by 3x (UTF-16) under NFC
    ret.push("\u{1D160}".to_string().into_bytes()); // expands by 3x (UTF-8) under NFC
    ret.push(vec![0xf4, 0x8f, 0xbf, 0xbe]); // illegal outside end of max range U+10FFFF
    ret.push(vec![239, 191, 191]); // 65535
    ret.push(vec![240, 144, 128, 128]); // 65536
    ret.push(vec![0xef, 0xbb, 0xbf]); // the canonical utf8 bom
    ret.push(vec![0xfe, 0xff]); // utf16 be bom
    ret.push(vec![0xff, 0xfe]); // utf16 le bom
    ret.push(vec![0, 0, 0xff, 0xff]); // ascii null be
    ret.push(vec![0xff, 0xff, 0, 0]); // ascii null le
    ret.push(vec![43, 47, 118, 56]); // and some others from wikipedia
    ret.push(vec![43, 47, 118, 57]);
    ret.push(vec![43, 47, 118, 43]);
    ret.push(vec![43, 47, 118, 47]);
    ret.push(vec![247, 100, 76]);
    ret.push(vec![221, 115, 102, 115]);
    ret.push(vec![14, 254, 255]);
    ret.push(vec![251, 238, 40]);
    ret.push(vec![251, 238, 40, 255]);
    ret.push(vec![132, 49, 149, 51]);

    let valid_points_and_ranges = vec![
        Range::Interval(0x0009, 0x000d),
        Range::Scalar(0x00a0),
        Range::Scalar(0x1680),
        Range::Scalar(0x180e),
        Range::Interval(0x2000, 0x200a),
        Range::Scalar(0x2028),
        Range::Scalar(0x2029),
        Range::Scalar(0x202f),
        Range::Scalar(0x205f),
        Range::Scalar(0x3000),
        Range::Interval(0x200e, 0x200f),
        Range::Interval(0x202a, 0x202e),
        Range::Interval(0x200c, 0x200d),
        Range::Scalar(0x0345),
        Range::Scalar(0x00b6),
        Range::Interval(0x02d0, 0x02d1),
        Range::Scalar(0xff70),
        Range::Interval(0x02b0, 0x02b8),
        Range::Scalar(0xfdd0),
        Range::Scalar(0x034f),
        Range::Interval(0x115f, 0x1160),
        Range::Interval(0x2065, 0x2069),
        Range::Scalar(0x3164),
        Range::Scalar(0xffa0),
        Range::Scalar(0xe0001),
        Range::Interval(0xe0020, 0xe007f),
        Range::Interval(0x0e40, 0x0e44),
        Range::Scalar(0x1f4a9), // poop emoji?
    ];

    // convert unicode code points and ranges to utf-8
    for i in &valid_points_and_ranges {
        match i {
            Range::Interval(lo, hi) => {
                for p in *lo..=*hi {
                    ret.push(char::from_u32(p).unwrap().to_string().into_bytes());
                }
            }
            Range::Scalar(p) => {
                ret.push(char::from_u32(*p).unwrap().to_string().into_bytes());
            }
        }
    }

    ret
});

pub fn sed_utf8_insert(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(rng);
    let data = data.expect("data is not None");
    let mut new_data = data.clone();
    let p = rng.gen_range(0..=data.len());
    let bytes = FUNNY_UNICODE.choose(rng).expect("choose() should not fail");
    for i in bytes.iter().rev() {
        // insert in reverse order
        new_data.insert(p, *i);
    }
    (Some(new_data), d)
}

pub fn sed_seq_repeat(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let data = data.expect("data is not None");
    if data.len() >= 2 {
        let mut new_data = Vec::new();
        let start = rng.gen_range(0..data.len() - 1);
        let end = rng.gen_range(start + 1..data.len());
        let pre = &data[0..start];
        let post = &data[end..data.len()];
        let stut = &data[start..end];
        let n = 10.rand_log(rng);
        let n = std::cmp::max(1024, n); // max 2^10 = 1024 stuts
        let d = rand_delta(rng);
        new_data.extend(pre);
        for _ in 0..n {
            new_data.extend(stut);
        }
        new_data.extend(post);
        (Some(new_data), d)
    } else {
        (Some(data.clone()), 0)
    }
}

pub fn sed_seq_del(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let data = data.expect("data is not None");
    let d = rand_delta(rng);
    let new_data = crate::generic::list_del_seq(rng, data.clone());
    (Some(new_data), d)
}

// Lines

fn get_lines(data: Option<&Vec<u8>>) -> Vec<Vec<u8>> {
    let mut lines: Vec<Vec<u8>> = vec![];
    let mut prev_index = 0;
    if let Some(data) = data {
        for (index, val) in data.iter().enumerate() {
            if *val == 10 {
                let end = match index {
                    _n if index < data.len() => index + 1,
                    _ => data.len(),
                };
                lines.push(data[prev_index..end].to_vec());
                prev_index = index + 1;
            }
        }
    }
    lines
}

fn try_lines(data: Option<&Vec<u8>>) -> Option<Vec<Vec<u8>>> {
    data?;
    let lines = get_lines(data);
    if let Some(first_line) = lines.first() {
        // first line (start of block) looks binary
        if is_binarish(Some(first_line)) {
            return None;
        }
        return Some(lines);
    }
    None
}

pub fn sed_line_del(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let data = data.expect("data is not None");
    if let Some(lines) = try_lines(Some(data)) {
        let new_data = crate::generic::list_del(rng, lines).concat();
        return (Some(new_data), 1);
    }
    (Some(data.clone()), -1)
}

pub fn sed_line_del_seq(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_del_seq(rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_dup(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_dup(rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_clone(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_clone(rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_swap(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_swap(rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_perm(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    debug!("sed_line_perm");
    let data = data.expect("data is not None");
    if let Some(lines) = try_lines(Some(data)) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_perm(rng, lines);
        debug!("new_lines {new_lines:?}");
        return (Some(new_lines.concat()), 1);
    }
    (Some(data.clone()), -1)
}

pub fn sed_line_repeat(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_repeat(rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}
pub fn sed_line_ins(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_ins(rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_replace(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_replace(rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_fuse_this(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = data {
        let new_data = crate::generic::list_fuse(rng, data, data);
        let d = rand_delta_up(rng);
        return (Some(new_data), d);
    }
    (None, 0)
}

pub fn sed_fuse_next(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = data {
        //split
        let (al1, al2) = data.split_at(data.len() / 2);
        let abl = crate::generic::list_fuse(rng, &al1.to_vec(), data);
        let abal = crate::generic::list_fuse(rng, &abl, &al2.to_vec());
        let d = rand_delta_up(rng);
        return (Some(abal), d);
    }
    (None, 0)
}

// Not a 1-1 with radamsa sed-fuse-old
pub fn sed_fuse_old(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = data {
        //split
        let (al1, al2) = data.split_at(data.len() / 2);
        let mut a = crate::generic::list_fuse(rng, &al1.to_vec(), &al2.to_vec());
        let mut b = crate::generic::list_fuse(rng, &al1.to_vec(), &al2.to_vec());
        a.append(&mut b);
        let d = rand_delta_up(rng);
        return (Some(a), d);
    }
    (None, 0)
}

// Tree mutations

pub fn sed_tree_del(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = data {
        let new_data = crate::split::sed_tree_op(rng, data, &crate::split::TreeMutate::TreeDel);
        if new_data.is_some() {
            return (new_data, 1);
        }
        return (new_data, -1);
    }
    (None, 0)
}

pub fn sed_tree_dup(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = data {
        let new_data = crate::split::sed_tree_op(rng, data, &crate::split::TreeMutate::TreeDup);
        if new_data.is_some() {
            return (new_data, 1);
        }
        return (new_data, -1);
    }
    (None, 0)
}

pub fn sed_tree_stutter(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = data {
        let new_data = crate::split::sed_tree_op(rng, data, &crate::split::TreeMutate::TreeStutter);
        if new_data.is_some() {
            return (new_data, 1);
        }
        return (new_data, -1);
    }
    (None, 0)
}

pub fn sed_tree_swap1(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = data {
        let new_data =
            crate::split::sed_tree_op(rng, data, &crate::split::TreeMutate::TreeSwapReplace);
        if new_data.is_some() {
            return (new_data, 1);
        }
        return (new_data, -1);
    }
    (None, 0)
}

pub fn sed_tree_swap2(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = data {
        let new_data =
            crate::split::sed_tree_op(rng, data, &crate::split::TreeMutate::TreeSwapPair);
        if new_data.is_some() {
            return (new_data, 1);
        }
        return (new_data, -1);
    }
    (None, 0)
}

mod ascii;

pub fn ascii_bad(rng: &mut dyn RngCore, data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = data {
        return match ascii::Ascii::parse(data) {
            Ok(mut cs) => {
                cs.mutate(rng);
                (Some(cs.unlex()), rand_delta_up(rng))
            }
            Err(()) => (Some(data.clone()), -1),
        };
    }
    (None, 0)
}

pub fn nop(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    debug!("test nop");
    (None, 0)
}

// Unit Tests

#[cfg(test)]
mod tests {
    use std::boxed::Box;

    use print_bytes::println_lossy;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    use super::*;

    #[test]
    fn test_get_num() {
        let num_1 = vec![48, 48, 48, 49, 10, 51]; // 0001
        let num_2 = vec![51, 50, 49, 10, 51]; // 321
        assert_eq!(get_num(Some(&num_1)), (Some(i256::from(1)), Some(4)));
        assert_eq!(get_num(Some(&num_2)), (Some(i256::from(321)), Some(3)));
    }

    #[test]
    fn test_mutate_a_num() {
        let num_1 = Box::from(vec![51, 50, 49, 32, 51, 10]); // 321
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (_which, data1) = mutate_a_num(&mut rng, Some(&num_1));
        let d1 = data1.as_ref().unwrap();
        assert_eq!(std::str::from_utf8(d1), Ok("321 1487970283344404796\n"));
        let mut rng2 = ChaCha20Rng::seed_from_u64(1_674_713_110);
        let (_which, data2) = mutate_a_num(&mut rng2, Some(&num_1));
        let d2 = data2.as_ref().unwrap();
        assert_eq!(
            std::str::from_utf8(d2),
            Ok("170141183460469231731687303715884105729 3\n")
        );
        let num_2: Vec<u8> = b"1 2 3 4 5 6 7\n".to_vec();
        let mut rng3 = ChaCha20Rng::seed_from_u64(1_674_713_145);
        let (_which, data3) = mutate_a_num(&mut rng3, Some(&num_2));
        let d3 = data3.as_ref().unwrap();
        assert_eq!(std::str::from_utf8(d3), Ok("1 2 3 4 -1 6 7\n"));
    }

    #[test]
    fn test_sed_num() {
        let num_1 = Vec::from(b"1 2 3 4 5 6 7 8 9 10 11 12\n"); // not-binary
        let num_2 = vec![255, 32, 129, 50, 49, 32, 51, 10]; // binary
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (_data, delta) = sed_num(&mut rng, Some(&num_1));
        assert_eq!(delta, 2);
        let (_data, delta) = sed_num(&mut rng, Some(&num_2));
        assert_eq!(delta, -1);
    }

    #[test]
    fn test_sed_byte_drop() {
        let data = Vec::from(b"ABCDEFG");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_byte_drop(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len() + 1,
            "Exactly one byte was dropped"
        );

        let data = Vec::from(b"");
        let (data, _delta) = sed_byte_drop(&mut rng, Some(&data));
        assert_eq!(
            data.expect("data is not None").len(),
            0,
            "Zero-sized input stays empty"
        );
    }

    #[test]
    fn test_sed_byte_inc() {
        let data = Vec::from(b"ABCDEFG");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_byte_inc(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len(),
            "Size did not change"
        );

        let data = Vec::from(b"");
        let (data, _delta) = sed_byte_inc(&mut rng, Some(&data));
        assert_eq!(
            data.expect("data is not None").len(),
            0,
            "Zero-sized input stays empty"
        );

        let data = Vec::from([0x41]);
        let (data, _delta) = sed_byte_inc(&mut rng, Some(&data));
        if let Some(data_new) = data {
            assert_eq!(data_new.len(), 1, "Size did not change");
            assert_eq!(data_new[0], 0x42, "Value was incremented");
        }
    }

    #[test]
    fn test_sed_byte_dec() {
        let data = Vec::from(b"ABCDEFG");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_byte_dec(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len(),
            "Size did not change"
        );

        let data = Vec::from(b"");
        let (data, _delta) = sed_byte_dec(&mut rng, Some(&data));
        assert_eq!(
            data.expect("data is not None").len(),
            0,
            "Zero-sized input stays empty"
        );

        let data = Vec::from([0x41]);
        let (data, _delta) = sed_byte_dec(&mut rng, Some(&data));
        if let Some(data_new) = data {
            assert_eq!(data_new.len(), 1, "Size did not change");
            assert_eq!(data_new[0], 0x40, "Value was decremented");
        }
    }

    #[test]
    fn test_sed_byte_flip() {
        let data = Vec::from(b"ABCDEFG");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_byte_flip(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len(),
            "Size did not change"
        );

        let data = Vec::from(b"");
        let (data, _delta) = sed_byte_flip(&mut rng, Some(&data));
        assert_eq!(
            data.expect("data is not None").len(),
            0,
            "Zero-sized input stays empty"
        );

        let data = Vec::from([0x41]);
        let (maybe_data, _delta) = sed_byte_flip(&mut rng, Some(&data));
        if let Some(data_new) = maybe_data {
            assert_eq!(data_new.len(), 1, "Size did not change");
            let b = data_new[0] ^ data[0];
            assert_eq!(b.count_ones(), 1, "Only a single bit was flipped");
        }
    }

    #[test]
    fn test_sed_byte_insert() {
        let data = Vec::from(b"ABCDEFG");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_byte_insert(&mut rng, Some(&data));
        assert_eq!(
            data.len() + 1,
            data2.expect("data2 is not None").len(),
            "Size increased by one"
        );

        let data = Vec::from(b"");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_byte_insert(&mut rng, Some(&data));
        assert_eq!(
            data2.expect("data2 is not None").len(),
            1,
            "Insert works on zero-sized data"
        );
    }

    #[test]
    fn test_sed_byte_repeat() {
        let data = Vec::from(b"AAAAAAAA");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_byte_repeat(&mut rng, Some(&data));
        assert!(
            data2.expect("data2 is not None").len() > data.len(),
            "Repeat increases the size of data"
        );
    }

    #[test]
    fn test_sed_byte_random() {
        let data = Vec::from(b"AAAAAAAA");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_byte_random(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len(),
            "Size of data does not increase"
        );
    }

    #[test]
    fn test_sed_byte_perm() {
        let data1 = Vec::from(b"ABCDEFGHI");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_byte_perm(&mut rng, Some(&data1));
        let Some(data2) = data2 else {
            panic!("data2 is not None");
        };
        assert_eq!(data1.len(), data2.len(), "Size of data does not increase");
        let adder = |a: usize, b: &u8| a + (*b as usize);
        let sum_data1 = data1.iter().fold(0, adder);
        let sum_data2 = data2.iter().fold(0, adder);
        assert_eq!(
            sum_data1, sum_data2,
            "Permutation does not change any bytes, just their ordering"
        );
    }

    #[test]
    fn test_sed_utf8_widen() {
        let data1 = Vec::from(b"1");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_utf8_widen(&mut rng, Some(&data1));
        let Some(data2) = data2 else {
            panic!("data2 is not None");
        };
        assert_eq!(data2.len(), 2, "utf8_widen widened a character");
        assert_eq!(data2[0], 0b1100_0000);
        assert_eq!(data2[1], data1[0] | 0b1000_0000);
    }

    #[test]
    fn test_sed_utf8_insert() {
        let data1 = Vec::from(b"");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_utf8_insert(&mut rng, Some(&data1));
        let Some(data2) = data2 else {
            panic!("data2 is not None");
        };
        let a = FUNNY_UNICODE.iter().find(|x| data2.as_slice() == **x);
        assert!(a.is_some());
        debug!("{a:?}");

        for i in FUNNY_UNICODE.iter() {
            debug!("{i:?}");
        }
    }

    #[test]
    fn test_sed_seq_repeat() {
        let data1 = Vec::from(b"ABCDEFGHIJ");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_seq_repeat(&mut rng, Some(&data1));
        let Some(data2) = data2 else {
            panic!("data2 is not None");
        };
        assert!(
            data2.len() >= data1.len(),
            "sed_seq_repeat increases size of data"
        );
    }

    #[test]
    fn test_sed_seq_del() {
        let data1 = Vec::from(b"ABCDEFGHIJ");
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (data2, _delta) = sed_seq_del(&mut rng, Some(&data1));
        let Some(data2) = data2 else {
            panic!("data2 is not None");
        };
        assert!(
            data2.len() <= data1.len(),
            "sed_seq_repeat reduces size of data"
        );
    }

    #[test]
    #[allow(clippy::naive_bytecount)]
    fn test_line_op() {
        let data1 = Vec::from(
            "ABCDE\nKLMNOPQRSTUV\nZYX\nfeklafnewlka\nkelwflknewfw\n123214324\nhello world\n"
                .as_bytes(),
        );
        //println_lossy(&data1);
        let mut data2 = Vec::from(b"ABCDEFGHIJ\nKLNOPQRSTUV\nZYX\n");
        data2.insert(5, 0xFE);
        data2.insert(6, 0xFE);
        data2.insert(7, 0xFE);
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let (_l, delta) = sed_line_del(&mut rng, Some(&data1));
        assert_eq!(1, delta);
        //debug!("{:?}", l);
        let (_l, delta) = sed_line_del(&mut rng, Some(&data2));
        assert_eq!(-1, delta);
        let (_l, delta) = sed_line_del_seq(&mut rng, Some(&data1));
        //debug!("{:?}", l);
        assert_eq!(1, delta);

        let (l, _delta) = sed_line_dup(&mut rng, Some(&data1));
        // println_lossy(&data1);
        // println_lossy(&l.unwrap());
        let og_count = data1.iter().filter(|&n| *n == 10u8).count();
        let new_count = l.unwrap().iter().filter(|&n| *n == 10u8).count();
        assert_eq!(og_count + 1, new_count);

        let (_l, _delta) = sed_line_clone(&mut rng, Some(&data1));
        let (_l, _delta) = sed_line_swap(&mut rng, Some(&data1));
        let (l, _delta) = sed_line_perm(&mut rng, Some(&data1));
        debug!("PERM:");
        println_lossy(&l.unwrap());
        let (_l, _delta) = sed_line_repeat(&mut rng, Some(&data1));
        let (_l, _delta) = sed_line_ins(&mut rng, Some(&data1));
        let (_l, _delta) = sed_line_replace(&mut rng, Some(&data1));
        //debug!("REPEAT:");
        //println_lossy(&l.unwrap());
    }

    #[test]
    fn test_sed_fuse_next() {
        let data1 = Vec::from(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\n");
        let mut rng = ChaCha20Rng::seed_from_u64(1_683_310_580);
        let (data2, _delta) = sed_fuse_next(&mut rng, Some(&data1));
        let expected = vec![65, 66, 67, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 10];
        assert_eq!(data2, Some(expected));
    }

    #[test]
    fn test_sed_fuse_old() {
        let data1 = Vec::from(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\n");
        let mut rng = ChaCha20Rng::seed_from_u64(1_683_310_580);
        let (data2, _delta) = sed_fuse_old(&mut rng, Some(&data1));
        let expected = vec![
            65, 66, 67, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 10, 65, 66, 67, 81, 82, 83, 84, 85,
            86, 87, 88, 89, 90, 10,
        ];
        assert_eq!(data2, Some(expected));
    }
}
