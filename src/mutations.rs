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

use rand::{seq::SliceRandom, Rng};
use std::collections::BTreeMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::shared::*;
use ethnum::*;
use rand::RngCore;

#[cfg(not(test))]
use log::debug;

#[cfg(test)]
use std::println as debug;

// pub const DEFAULT_MUTATIONS: &'static str = "ft=2,fo=2,fn,num=5,td,tr2,ts1,tr,ts2,ld,lds,lr2,li,ls,lp,lr,lis,lrs,sr,sd,bd,bf,bi,br,bp,bei,bed,ber,uw,ui=2,xp=9,ab";
pub const DEFAULT_MUTATIONS: &'static str =
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

impl MutaType {
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
    fn mutate(&self, _rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
        use MutaType::*;
        match *self {
            AsciiBad => ascii_bad(_rng, _data),
            ByteDrop => sed_byte_drop(_rng, _data),
            ByteFlip => sed_byte_flip(_rng, _data),
            ByteInsert => sed_byte_insert(_rng, _data),
            ByteRepeat => sed_byte_repeat(_rng, _data),
            BytePerm => sed_byte_perm(_rng, _data),
            ByteInc => sed_byte_inc(_rng, _data),
            ByteDec => sed_byte_dec(_rng, _data),
            ByteRand => sed_byte_random(_rng, _data),
            SeqRepeat => sed_seq_repeat(_rng, _data),
            SeqDel => sed_seq_del(_rng, _data),
            LineDel => sed_line_del(_rng, _data),
            LineDelSeq => sed_line_del_seq(_rng, _data),
            LineDup => sed_line_dup(_rng, _data),
            LineClone => sed_line_clone(_rng, _data),
            LineRepeat => sed_line_repeat(_rng, _data),
            LineSwap => sed_line_swap(_rng, _data),
            LinePerm => sed_line_perm(_rng, _data),
            LineIns => sed_line_ins(_rng, _data),
            LineReplace => sed_line_replace(_rng, _data),
            TreeDel => sed_tree_del(_rng, _data),
            TreeDup => sed_tree_dup(_rng, _data),
            TreeSwap1 => sed_tree_swap1(_rng, _data),
            TreeSwap2 => sed_tree_swap2(_rng, _data),
            TreeRepeat => sed_tree_stutter(_rng, _data),
            UTF8Widen => sed_utf8_widen(_rng, _data),
            UTF8Insert => sed_utf8_insert(_rng, _data),
            Num => sed_num(_rng, _data),
            Str => (None, 0),
            Word => (None, 0),
            Xp => (None, 0),
            FuseThis => sed_fuse_this(_rng, _data),
            FuseNext => sed_fuse_next(_rng, _data),
            FuseOld => sed_fuse_old(_rng, _data),
            Nop => nop(_rng, _data),
        }
    }
    pub fn id_to_mutatype(_id: &str) -> Option<MutaType> {
        let mut mi = MutaType::iter();
        while let Some(muta) = mi.next() {
            if muta.id() == _id {
                return Some(muta);
            }
        }
        None
    }
}

pub fn init_mutations() -> BTreeMap<MutaType, Mutator> {
    let mut map = BTreeMap::<MutaType, Mutator>::new();
    let mut mi = MutaType::iter();
    while let Some(muta) = mi.next() {
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
            .finish()
    }
}

impl Mutator {
    pub fn new(_muta: MutaType) -> Mutator {
        Mutator {
            muta: _muta,
            priority: 0,
            score: MAX_SCORE,
            weight: 0,
            delta: 0,
        }
    }
    pub fn id(&self) -> String {
        self.muta.id()
    }
    pub fn info(&self) -> String {
        self.muta.info()
    }
}

impl Mutations {
    pub fn new() -> Mutations {
        Mutations {
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
    pub fn randomize(&mut self, _rng: &mut dyn RngCore) {
        if self.mutas.is_some() {
            // Apply random scores
            for (_, mutator) in self.mutators.iter_mut() {
                if self.mutator_nodes.contains(&mutator.muta) {
                    let rand_value = MAX_SCORE.rands(_rng);
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

    fn weighted_permutation(&mut self, _rng: &mut dyn RngCore) -> Vec<&mut Mutator> {
        let mut out_mutas: Vec<&mut Mutator> = vec![];
        for (_, m) in self.mutators.iter_mut() {
            if let Some(mutas) = &self.mutas {
                if mutas.contains(&m.muta) {
                    if m.priority > 0 {
                        m.weight = (m.priority * m.score).rands(_rng);
                        out_mutas.push(m);
                    }
                }
            } else if self.mutator_nodes.contains(&m.muta) {
                if m.priority > 0 {
                    m.weight = (m.priority * m.score).rands(_rng);
                    out_mutas.push(m);
                }
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
        _rng: &mut dyn RngCore,
        _data: Option<&Vec<u8>>,
    ) -> Option<Vec<u8>> {
        let mut mutas = self.weighted_permutation(_rng);
        let data = _data?;
        while !mutas.is_empty() {
            let mut muta = mutas.pop()?;
            debug!("muta {}", muta.id());
            match muta.muta.mutate(_rng, Some(data)) {
                (Some(new_data), delta) => {
                    // always remember whatever was learned
                    muta.score = adjust_priority(muta.score, delta);
                    muta.delta = delta;
                    if new_data != *data {
                        return Some(new_data);
                    } else {
                        debug!("Nothing changed");
                    }
                }
                _ => {
                    debug!("Nothing changed");
                }
            }
        }
        _data.cloned()
    }
}

/// This function parses mutation string i.e. ft=2,fo=2
pub fn string_mutators(_input: &str, _mutators: &mut BTreeMap<MutaType, Mutator>) -> Vec<MutaType> {
    let mut applied_mutators: Vec<MutaType> = vec![];
    let string_list = _input.trim().split(",").collect::<Vec<&str>>();
    //debug!("mutators {:#?}", _mutators);
    for s in string_list {
        let tuple = s.trim().split("=").collect::<Vec<&str>>();
        let mutator_id = tuple.get(0).unwrap_or(&"").trim();
        let priority = tuple
            .get(1)
            .unwrap_or(&"0")
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        let muta_type = MutaType::id_to_mutatype(mutator_id).unwrap_or(MutaType::Nop);
        if let Some(mutator) = _mutators.get_mut(&muta_type) {
            mutator.priority = if priority < 1 { 1 } else { priority };
            mutator.score = MAX_SCORE;
            applied_mutators.push(mutator.muta);
        } else {
            panic!("unknown mutator {}", mutator_id);
        }
    }
    applied_mutators
}

fn rand_delta(_rng: &mut dyn RngCore) -> isize {
    if _rng.gen() {
        1
    } else {
        -1
    }
}

fn rand_delta_up(_rng: &mut dyn RngCore) -> isize {
    // slight positive bias
    if _rng.gen_range(0..20) <= 11 {
        1
    } else {
        -1
    }
}

fn adjust_priority(_pri: usize, _delta: isize) -> usize {
    match _delta {
        0 => _pri,
        _ => {
            (std::cmp::max(
                MIN_SCORE as isize,
                std::cmp::max(MAX_SCORE as isize, _pri as isize + _delta),
            )) as usize
        }
    }
}

/// Number Mutators

// get digit
fn get_num(_data: Option<&[u8]>) -> (Option<i256>, Option<usize>) {
    let mut out = vec![];
    let mut n = i256::from(0);
    if let Some(data) = _data {
        for val in data.iter() {
            if let Some(_) = char::from(*val).to_digit(10) {
                out.push(*val);
            } else {
                break;
            }
        }
        if out.len() == 0 {
            if data.len() > 0 {
                return (None, Some(1));
            } else {
                return (None, None);
            }
        }
        for (pos, m) in out.iter().rev().enumerate() {
            let num = i256::from(char::from(*m).to_digit(10).unwrap_or(0));
            if pos == 0 {
                n = num;
            } else {
                n += num
                    .overflowing_mul(i256::from((10_usize.overflowing_pow(pos as u32).0) as i128))
                    .0;
            }
        }
        return (Some(n), Some(out.len()));
    }
    (None, None)
}
fn mutate_a_num(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (isize, Option<Vec<u8>>) {
    let mut offset = 0_usize;
    let mut nfound = 0_usize;
    let mut _which = 0_isize;
    if let Some(data) = _data {
        if data.len() == 0 {
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
        _which = match nfound {
            0 => return (0, None),
            _ => nfound.rands(_rng) as isize,
        };
        if let Some(target) = num_offsets.get(_which as usize) {
            let new_num = mutate_num(_rng, target.0).to_string().as_bytes().to_vec();
            let mut new_lst = data[..target.1].to_vec();
            new_lst.extend(new_num);
            new_lst.extend(&data[target.1 + target.2..]);
            return (_which + 1, Some(new_lst));
        }
    }
    (0, None)
}

// -> lst, delta
pub fn sed_num(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let (which, data) = mutate_a_num(_rng, _data);
    // check if binary
    let is_bin = match data {
        Some(ref d) => is_binarish(Some(d)),
        None => false,
    };
    if which == 0 {
        //textual with less frequent numbers
        debug!("textual with less numbers");
        let n = 10_usize.rands(_rng);
        if n == 0 {
            return (data, -1);
        } else {
            return (data, 0);
        }
    } else if is_bin {
        return (data, -1);
    }
    (data, 2)
}

// Byte-level Mutations

pub fn sed_byte_drop(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    if data.len() > 0 {
        let p = _rng.gen_range(0..data.len());
        new_data.remove(p);
    }
    (Some(new_data), d)
}

pub fn sed_byte_inc(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    if data.len() > 0 {
        let p = _rng.gen_range(0..data.len());
        new_data[p] = new_data[p].wrapping_add(1);
    }
    (Some(new_data), d)
}

pub fn sed_byte_dec(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    if data.len() > 0 {
        let p = _rng.gen_range(0..data.len());
        new_data[p] = new_data[p].wrapping_sub(1);
    }
    (Some(new_data), d)
}

pub fn sed_byte_flip(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    if data.len() > 0 {
        let b = 1 << _rng.gen_range(0..8);
        let p = _rng.gen_range(0..data.len());
        new_data[p] ^= b;
    }
    (Some(new_data), d)
}

pub fn sed_byte_insert(
    _rng: &mut dyn RngCore,
    _data: Option<&Vec<u8>>,
) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    let b = _rng.gen::<u8>();
    let p = _rng.gen_range(0..=data.len());
    new_data.insert(p, b);
    (Some(new_data), d)
}

fn repeat_len(_rng: &mut dyn RngCore) -> usize {
    let mut limit = 0b10;
    while _rng.gen() && limit != 0x20000 {
        limit <<= 1;
    }
    _rng.gen_range(0..limit)
}

pub fn sed_byte_repeat(
    _rng: &mut dyn RngCore,
    _data: Option<&Vec<u8>>,
) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let n = repeat_len(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    if data.len() > 0 {
        let p = _rng.gen_range(0..data.len());
        let to_repeat = data[p];
        for _ in 0..n {
            // repeat data[p] n times
            new_data.insert(p, to_repeat);
        }
    }
    (Some(new_data), d)
}

pub fn sed_byte_random(
    _rng: &mut dyn RngCore,
    _data: Option<&Vec<u8>>,
) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    if data.len() > 0 {
        let b = _rng.gen::<u8>();
        let p = _rng.gen_range(0..data.len());
        new_data[p] = b;
    }
    (Some(new_data), d)
}

pub fn sed_byte_perm(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    if data.len() > 0 {
        let p = _rng.gen_range(0..data.len());
        let n = std::cmp::min(p + _rng.gen_range(2..20), new_data.len());
        new_data[p..n].shuffle(_rng);
    }
    (Some(new_data), d)
}

pub fn sed_utf8_widen(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    if data.len() > 0 {
        let p = _rng.gen_range(0..data.len());
        // assuming we hit a 6-bit ascii char, make it unnecessarily wide
        // which might confuse a length calculation
        let b = new_data[p];
        if b == (b & 0b111111) {
            new_data[p] = 0b11000000;
            new_data.insert(p + 1, b | 0b10000000);
        }
    }
    (Some(new_data), d)
}

lazy_static! {
    static ref FUNNY_UNICODE: Vec< Vec<u8> > = {
        let mut ret = Vec::new();
        ret.push("\u{202E}".to_string().into_bytes());  // Right to Left Override
        ret.push("\u{202D}".to_string().into_bytes());  // Left to Right Override
        ret.push("\u{180E}".to_string().into_bytes());  // Mongolian Vowel Separator
        ret.push("\u{2060}".to_string().into_bytes());  // Word Joiner
        ret.push("\u{FEFE}".to_string().into_bytes());  // reserved
        ret.push("\u{FFFF}".to_string().into_bytes());  // not a character
        ret.push("\u{0FED}".to_string().into_bytes());  // unassigned
        ret.push(vec![0xed, 0xba, 0xad]);               // U+DEAD illegal low surrogate
        ret.push(vec![0xed, 0xaa, 0xad]);               // U+DAAD illegal high surrogate
        ret.push("\u{F8FF}".to_string().into_bytes());  // private use char (Apple)
        ret.push("\u{FF0F}".to_string().into_bytes());  // full width solidus
        ret.push("\u{1D7D6}".to_string().into_bytes()); // MATHEMATICAL BOLD DIGIT EIGHT
        ret.push("\u{00DF}".to_string().into_bytes());  // IDNA deviant
        ret.push("\u{FDFD}".to_string().into_bytes());  // expands by 11x (UTF-8) and 18x (UTF-16) NFKC
        ret.push("\u{0390}".to_string().into_bytes());  // expands by 3x (UTF-8) NFD
        ret.push("\u{1F82}".to_string().into_bytes());  // expands by 4x (UTF-16) NFD
        ret.push("\u{FB2C}".to_string().into_bytes());  // expands by 3x (UTF-16) under NFC
        ret.push("\u{1D160}".to_string().into_bytes()); // expands by 3x (UTF-8) under NFC
        ret.push(vec![0xf4, 0x8f, 0xbf, 0xbe]);         // illegal outside end of max range U+10FFFF
        ret.push(vec![239, 191, 191]);                  // 65535
        ret.push(vec![240, 144, 128, 128]);             // 65536
        ret.push(vec![0xef, 0xbb, 0xbf]);               // the canonical utf8 bom
        ret.push(vec![0xfe, 0xff]);                     // utf16 be bom
        ret.push(vec![0xff, 0xfe]);                     // utf16 le bom
        ret.push(vec![0, 0, 0xff, 0xff]);               // ascii null be
        ret.push(vec![0xff, 0xff, 0, 0]);               // ascii null le
        ret.push(vec![43, 47, 118, 56]);                // and some others from wikipedia
        ret.push(vec![43, 47, 118, 57]);
        ret.push(vec![43, 47, 118, 43]);
        ret.push(vec![43, 47, 118, 47]);
        ret.push(vec![247, 100, 76]);
        ret.push(vec![221, 115, 102, 115]);
        ret.push(vec![14, 254, 255]);
        ret.push(vec![251, 238, 40]);
        ret.push(vec![251, 238, 40, 255]);
        ret.push(vec![132, 49, 149, 51]);

        enum Range {
            Interval(u32, u32),
            Scalar(u32)
        }

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
        for i in valid_points_and_ranges.iter() {
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
    };
}

pub fn sed_utf8_insert(
    _rng: &mut dyn RngCore,
    _data: Option<&Vec<u8>>,
) -> (Option<Vec<u8>>, isize) {
    let d = rand_delta(_rng);
    let data = _data.expect("_data is not None");
    let mut new_data = data.to_vec();
    let p = _rng.gen_range(0..=data.len());
    let bytes = FUNNY_UNICODE
        .choose(_rng)
        .expect("choose() should not fail");
    for i in bytes.iter().rev() {
        // insert in reverse order
        new_data.insert(p, *i);
    }
    (Some(new_data), d)
}

pub fn sed_seq_repeat(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let data = _data.expect("_data is not None");
    if data.len() >= 2 {
        let mut new_data = Vec::new();
        let start = _rng.gen_range(0..data.len() - 1);
        let end = _rng.gen_range(start + 1..data.len());
        let pre = &data[0..start];
        let post = &data[end..data.len()];
        let stut = &data[start..end];
        let n = 10.rand_log(_rng);
        let n = std::cmp::max(1024, n); // max 2^10 = 1024 stuts
        let d = rand_delta(_rng);
        new_data.extend(pre);
        for _ in 0..n {
            new_data.extend(stut);
        }
        new_data.extend(post);
        (Some(new_data), d)
    } else {
        (Some(data.to_vec()), 0)
    }
}

pub fn sed_seq_del(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let data = _data.expect("_data is not None");
    let d = rand_delta(_rng);
    let new_data = crate::generic::list_del_seq(_rng, data.to_vec());
    (Some(new_data), d)
}

// Lines

fn get_lines(_data: Option<&Vec<u8>>) -> Option<Vec<Vec<u8>>> {
    let mut lines: Vec<Vec<u8>> = vec![];
    let mut prev_index = 0;
    if let Some(data) = _data {
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
    Some(lines)
}

fn try_lines(_data: Option<&Vec<u8>>) -> Option<Vec<Vec<u8>>> {
    if _data.is_none() {
        return None;
    }
    if let Some(lines) = get_lines(_data) {
        if let Some(first_line) = lines.first() {
            // first line (start of block) looks binary
            if is_binarish(Some(&first_line)) {
                return None;
            }
            return Some(lines);
        }
    }
    None
}

pub fn sed_line_del(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    let data = _data.expect("_data is not None");
    if let Some(lines) = try_lines(Some(data)) {
        let new_data = crate::generic::list_del(_rng, lines).concat();
        return (Some(new_data), 1);
    }
    (Some(data.to_vec()), -1)
}

pub fn sed_line_del_seq(
    _rng: &mut dyn RngCore,
    _data: Option<&Vec<u8>>,
) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(_data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_del_seq(_rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_dup(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(_data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_dup(_rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_clone(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(_data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_clone(_rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_swap(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(_data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_swap(_rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_perm(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    debug!("sed_line_perm");
    let data = _data.expect("_data is not None");
    if let Some(lines) = try_lines(Some(data)) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_perm(_rng, lines);
        debug!("new_lines {:?}", new_lines);
        return (Some(new_lines.concat()), 1);
    }
    (Some(data.to_vec()), -1)
}

pub fn sed_line_repeat(
    _rng: &mut dyn RngCore,
    _data: Option<&Vec<u8>>,
) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(_data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_repeat(_rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}
pub fn sed_line_ins(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(_data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_ins(_rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_line_replace(
    _rng: &mut dyn RngCore,
    _data: Option<&Vec<u8>>,
) -> (Option<Vec<u8>>, isize) {
    if let Some(lines) = try_lines(_data) {
        let new_lines: Vec<Vec<u8>> = crate::generic::list_replace(_rng, lines);
        return (Some(new_lines.concat()), 1);
    }
    (None, -1)
}

pub fn sed_fuse_this(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = _data {
        let new_data = crate::generic::list_fuse(_rng, &data, &data);
        let d = rand_delta_up(_rng);
        return (Some(new_data), d);
    }
    (None, 0)
}

pub fn sed_fuse_next(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = _data {
        //split
        let (al1, al2) = data.split_at(data.len() / 2);
        let abl = crate::generic::list_fuse(_rng, &al1.to_vec(), &data);
        let abal = crate::generic::list_fuse(_rng, &abl, &al2.to_vec());
        let d = rand_delta_up(_rng);
        return (Some(abal), d);
    }
    (None, 0)
}

// Not a 1-1 with radamsa sed-fuse-old
pub fn sed_fuse_old(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = _data {
        //split
        let (al1, al2) = data.split_at(data.len() / 2);
        let mut a = crate::generic::list_fuse(_rng, &al1.to_vec(), &al2.to_vec());
        let mut b = crate::generic::list_fuse(_rng, &al1.to_vec(), &al2.to_vec());
        a.append(&mut b);
        let d = rand_delta_up(_rng);
        return (Some(a), d);
    }
    (None, 0)
}

// Tree mutations

pub fn sed_tree_del(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = _data {
        let new_data = crate::split::sed_tree_op(_rng, data, crate::split::TreeMutate::TreeDel);
        if new_data.is_some() {
            return (new_data, 1);
        } else {
            return (new_data, -1);
        }
    }
    (None, 0)
}

pub fn sed_tree_dup(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = _data {
        let new_data = crate::split::sed_tree_op(_rng, data, crate::split::TreeMutate::TreeDup);
        if new_data.is_some() {
            return (new_data, 1);
        } else {
            return (new_data, -1);
        }
    }
    (None, 0)
}

pub fn sed_tree_stutter(
    _rng: &mut dyn RngCore,
    _data: Option<&Vec<u8>>,
) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = _data {
        let new_data = crate::split::sed_tree_op(_rng, data, crate::split::TreeMutate::TreeStutter);
        if new_data.is_some() {
            return (new_data, 1);
        } else {
            return (new_data, -1);
        }
    }
    (None, 0)
}

pub fn sed_tree_swap1(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = _data {
        let new_data =
            crate::split::sed_tree_op(_rng, data, crate::split::TreeMutate::TreeSwapReplace);
        if new_data.is_some() {
            return (new_data, 1);
        } else {
            return (new_data, -1);
        }
    }
    (None, 0)
}

pub fn sed_tree_swap2(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = _data {
        let new_data =
            crate::split::sed_tree_op(_rng, data, crate::split::TreeMutate::TreeSwapPair);
        if new_data.is_some() {
            return (new_data, 1);
        } else {
            return (new_data, -1);
        }
    }
    (None, 0)
}

mod ascii;

pub fn ascii_bad(_rng: &mut dyn RngCore, _data: Option<&Vec<u8>>) -> (Option<Vec<u8>>, isize) {
    if let Some(data) = _data {
        let mut cs = ascii::Ascii::lex(data);
        if cs.first_block_has_text() {
            cs.mutate(_rng);
            return (Some(cs.unlex()), rand_delta_up(_rng));
        } else {
            return (Some(data.clone()), -1);
        }
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
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use print_bytes::println_lossy;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use std::boxed::Box;

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
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (_which, data1) = mutate_a_num(&mut rng, Some(&num_1));
        let d1 = data1.as_ref().unwrap();
        assert_eq!(std::str::from_utf8(&d1), Ok("321 1487970283344404796\n"));
        let mut rng2 = ChaCha20Rng::seed_from_u64(1674713110);
        let (_which, data2) = mutate_a_num(&mut rng2, Some(&num_1));
        let d2 = data2.as_ref().unwrap();
        assert_eq!(
            std::str::from_utf8(&d2),
            Ok("170141183460469231731687303715884105729 3\n")
        );
        let num_2: Vec<u8> = "1 2 3 4 5 6 7\n".as_bytes().to_vec();
        let mut rng3 = ChaCha20Rng::seed_from_u64(1674713145);
        let (_which, data3) = mutate_a_num(&mut rng3, Some(&num_2));
        let d3 = data3.as_ref().unwrap();
        assert_eq!(std::str::from_utf8(&d3), Ok("1 2 3 4 -1 6 7\n"));
    }
    #[test]
    fn test_sed_num() {
        let num_1 = Vec::from("1 2 3 4 5 6 7 8 9 10 11 12\n".as_bytes()); // not-binary
        let num_2 = vec![255, 32, 129, 50, 49, 32, 51, 10]; // binary
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (_data, delta) = sed_num(&mut rng, Some(&num_1));
        assert_eq!(delta, 2);
        let (_data, delta) = sed_num(&mut rng, Some(&num_2));
        assert_eq!(delta, -1);
    }

    #[test]
    fn test_sed_byte_drop() {
        let data = Vec::from("ABCDEFG".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_byte_drop(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len() + 1,
            "Exactly one byte was dropped"
        );

        let data = Vec::from("".as_bytes());
        let (_data, _delta) = sed_byte_drop(&mut rng, Some(&data));
        assert_eq!(
            _data.expect("_data is not None").len(),
            0,
            "Zero-sized input stays empty"
        );
    }

    #[test]
    fn test_sed_byte_inc() {
        let data = Vec::from("ABCDEFG".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_byte_inc(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len(),
            "Size did not change"
        );

        let data = Vec::from("".as_bytes());
        let (_data, _delta) = sed_byte_inc(&mut rng, Some(&data));
        assert_eq!(
            _data.expect("_data is not None").len(),
            0,
            "Zero-sized input stays empty"
        );

        let data = Vec::from([0x41]);
        let (_data, _delta) = sed_byte_inc(&mut rng, Some(&data));
        if let Some(data_new) = _data {
            assert_eq!(data_new.len(), 1, "Size did not change");
            assert_eq!(data_new[0], 0x42, "Value was incremented");
        }
    }

    #[test]
    fn test_sed_byte_dec() {
        let data = Vec::from("ABCDEFG".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_byte_dec(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len(),
            "Size did not change"
        );

        let data = Vec::from("".as_bytes());
        let (_data, _delta) = sed_byte_dec(&mut rng, Some(&data));
        assert_eq!(
            _data.expect("_data is not None").len(),
            0,
            "Zero-sized input stays empty"
        );

        let data = Vec::from([0x41]);
        let (_data, _delta) = sed_byte_dec(&mut rng, Some(&data));
        if let Some(data_new) = _data {
            assert_eq!(data_new.len(), 1, "Size did not change");
            assert_eq!(data_new[0], 0x40, "Value was decremented");
        }
    }

    #[test]
    fn test_sed_byte_flip() {
        let data = Vec::from("ABCDEFG".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_byte_flip(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len(),
            "Size did not change"
        );

        let data = Vec::from("".as_bytes());
        let (_data, _delta) = sed_byte_flip(&mut rng, Some(&data));
        assert_eq!(
            _data.expect("_data is not None").len(),
            0,
            "Zero-sized input stays empty"
        );

        let data = Vec::from([0x41]);
        let (_data, _delta) = sed_byte_flip(&mut rng, Some(&data));
        if let Some(data_new) = _data {
            assert_eq!(data_new.len(), 1, "Size did not change");
            let b = data_new[0] ^ data[0];
            assert_eq!(b.count_ones(), 1, "Only a single bit was flipped");
        }
    }

    #[test]
    fn test_sed_byte_insert() {
        let data = Vec::from("ABCDEFG".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_byte_insert(&mut rng, Some(&data));
        assert_eq!(
            data.len() + 1,
            data2.expect("data2 is not None").len(),
            "Size increased by one"
        );

        let data = Vec::from("".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_byte_insert(&mut rng, Some(&data));
        assert_eq!(
            data2.expect("data2 is not None").len(),
            1,
            "Insert works on zero-sized data"
        );
    }

    #[test]
    fn test_sed_byte_repeat() {
        let data = Vec::from("AAAAAAAA".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_byte_repeat(&mut rng, Some(&data));
        assert!(
            data2.expect("data2 is not None").len() > data.len(),
            "Repeat increases the size of data"
        );
    }

    #[test]
    fn test_sed_byte_random() {
        let data = Vec::from("AAAAAAAA".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_byte_random(&mut rng, Some(&data));
        assert_eq!(
            data.len(),
            data2.expect("data2 is not None").len(),
            "Size of data does not increase"
        );
    }

    #[test]
    fn test_sed_byte_perm() {
        let data1 = Vec::from("ABCDEFGHI".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
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
        let data1 = Vec::from("1".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_utf8_widen(&mut rng, Some(&data1));
        let Some(data2) = data2 else {
            panic!("data2 is not None");
        };
        assert_eq!(data2.len(), 2, "utf8_widen widened a character");
        assert_eq!(data2[0], 0b11000000);
        assert_eq!(data2[1], data1[0] | 0b10000000);
    }

    #[test]
    fn test_sed_utf8_insert() {
        let data1 = Vec::from("".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (data2, _delta) = sed_utf8_insert(&mut rng, Some(&data1));
        let Some(data2) = data2 else {
            panic!("data2 is not None");
        };
        let a = FUNNY_UNICODE.iter().find(|x| data2.as_slice() == **x);
        assert!(a.is_some());
        debug!("{:?}", a);

        for i in FUNNY_UNICODE.iter() {
            debug!("{:?}", i);
        }
    }

    #[test]
    fn test_sed_seq_repeat() {
        let data1 = Vec::from("ABCDEFGHIJ".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
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
        let data1 = Vec::from("ABCDEFGHIJ".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
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
    fn test_line_op() {
        let data1 = Vec::from(
            "ABCDE\nKLMNOPQRSTUV\nZYX\nfeklafnewlka\nkelwflknewfw\n123214324\nhello world\n"
                .as_bytes(),
        );
        //println_lossy(&data1);
        let mut data2 = Vec::from("ABCDEFGHIJ\nKLNOPQRSTUV\nZYX\n".as_bytes());
        data2.insert(5, 0xFE);
        data2.insert(6, 0xFE);
        data2.insert(7, 0xFE);
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let (_l, delta) = sed_line_del(&mut rng, Some(&data1));
        assert_eq!(1, delta);
        //debug!("{:?}", l);
        let (_l, delta) = sed_line_del(&mut rng, Some(&data2));
        assert_eq!(-1, delta);
        let (_l, delta) = sed_line_del_seq(&mut rng, Some(&data1));
        //debug!("{:?}", l);
        assert_eq!(1, delta);

        let (_l, _delta) = sed_line_dup(&mut rng, Some(&data1));
        // println_lossy(&data1);
        // println_lossy(&l.unwrap());
        let og_count = data1.iter().filter(|&n| *n == 10u8).count();
        let new_count = _l.unwrap().iter().filter(|&n| *n == 10u8).count();
        assert_eq!(og_count + 1, new_count);

        let (_l, _delta) = sed_line_clone(&mut rng, Some(&data1));
        let (_l, _delta) = sed_line_swap(&mut rng, Some(&data1));
        let (_l, _delta) = sed_line_perm(&mut rng, Some(&data1));
        debug!("PERM:");
        println_lossy(&_l.unwrap());
        let (_l, _delta) = sed_line_repeat(&mut rng, Some(&data1));
        let (_l, _delta) = sed_line_ins(&mut rng, Some(&data1));
        let (_l, _delta) = sed_line_replace(&mut rng, Some(&data1));
        //debug!("REPEAT:");
        //println_lossy(&l.unwrap());
    }

    #[test]
    fn test_sed_fuse_next() {
        let data1 = Vec::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ\n".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1683310580);
        let (data2, _delta) = sed_fuse_next(&mut rng, Some(&data1));
        let expected = vec![65, 66, 67, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 10];
        assert_eq!(data2, Some(expected));
    }

    #[test]
    fn test_sed_fuse_old() {
        let data1 = Vec::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ\n".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(1683310580);
        let (data2, _delta) = sed_fuse_old(&mut rng, Some(&data1));
        let expected = vec![
            65, 66, 67, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 10, 65, 66, 67, 81, 82, 83, 84, 85,
            86, 87, 88, 89, 90, 10,
        ];
        assert_eq!(data2, Some(expected));
    }
}
