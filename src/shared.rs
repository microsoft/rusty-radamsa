//! Utility functions for the crate.
//!
use ethnum::*;
use fraction::Fraction;
use log::*;
use rand::{Rng, RngCore};
use regex::Regex;
use std::path::Path;
use std::time::SystemTime;
use wax::{Glob, GlobError};

pub const AVG_BLOCK_SIZE: usize = 2048;
pub const MIN_BLOCK_SIZE: usize = 256;
pub const INITIAL_IP: usize = 24;
pub const MAX_BLOCK_SIZE: usize = 2 * AVG_BLOCK_SIZE;
pub const REMUTATE_PROBABILITY: f64 = 0.8; // 4/5
pub const MAX_CHECKSUM_RETRY: usize = 10000;
pub const MAX_UDP_PACKET_SIZE: usize = 65507;
pub const SILLY_STRINGS: [&'static str; 2] = ["cmd.exe", "/C"];

macro_rules! _vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

pub(crate) use _vec_of_strings;

pub(crate) fn time_seed() -> u64 {
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Duration since UNIX_EPOCH failed");
    d.as_secs()
}
pub trait Rands {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self;
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self;
}

pub(crate) fn safe_gen_range(_rng: &mut dyn RngCore, low: usize, high: usize) -> usize {
    if high == 0 {
        return high;
    }
    if low >= high {
        return high;
    }
    _rng.gen_range(low..high)
}

impl Rands for usize {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        _rng.gen_range(0..*self)
    }
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = _rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_usize.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(_rng);
            return val | hi;
        }
        0
    }
}

impl Rands for u64 {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        _rng.gen_range(0..*self)
    }
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = _rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_usize.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(_rng);
            return (val | hi) as u64;
        }
        0
    }
}

impl Rands for u128 {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        _rng.gen_range(0..*self)
    }
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = _rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_usize.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(_rng);
            return (val | hi) as u128;
        }
        0
    }
}

impl Rands for isize {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        _rng.gen_range(-*self..*self)
    }
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = _rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_isize.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(_rng);
            return val | hi;
        }
        0
    }
}

impl Rands for i128 {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        _rng.gen_range(-*self..*self)
    }
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = _rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_i128.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(_rng);
            return val | hi;
        }
        0
    }
}

impl Rands for i32 {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        _rng.gen_range(-*self..*self)
    }
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = _rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_i32.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(_rng);
            return val | hi;
        }
        0
    }
}

impl Rands for i64 {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        _rng.gen_range(-*self..*self)
    }
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = _rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_i64.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(_rng);
            return val | hi;
        }
        0
    }
}

impl Rands for i256 {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self {
        if *self == I256::from(0) {
            return I256::from(0);
        }
        if (self.as_i128().overflowing_abs().0 == I256::from(0)
            && self.as_i128().overflowing_neg().0 == I256::from(0))
            || self.as_i128().overflowing_abs().0 == self.as_i128().overflowing_neg().0
        {
            return I256::from(0);
        }
        I256::from(
            _rng.gen_range(self.as_i128().overflowing_neg().0..self.as_i128().overflowing_abs().0),
        )
    }
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = self.rands(_rng);
            if n == 0 {
                return I256::from(0);
            }
            let hi = I256::from(1).overflowing_shl(n.as_u32() - 1).0;
            let val = hi.rands(_rng);
            return val | hi;
        }
        I256::from(0)
    }
}

impl Rands for u256 {
    fn rands(&self, _rng: &mut dyn RngCore) -> Self {
        if *self == U256::from(0_u32) {
            return U256::from(0_u32);
        }
        U256::from(_rng.gen_range(0..self.as_u128()))
    }
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = self.rands(_rng);
            if n == 0 {
                return u256::from(0_u32);
            }
            let hi = u256::from(1_u32).overflowing_shl(n.as_u32() - 1).0;
            let val = hi.rands(_rng);
            return val | hi;
        }
        u256::from(0_u32)
    }
}

fn interesting_numbers() -> Vec<i256> {
    let nums: Vec<u32> = vec![1, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128];
    let mut out: Vec<i256> = vec![];
    for n in nums {
        let (x, is_overflow) = I256::from(1).overflowing_shl(n);
        if !is_overflow {
            out.push(x);
            out.push(x.overflowing_sub(I256::from(1)).0 as i256);
            out.push(x.overflowing_add(I256::from(1)).0 as i256);
        }
    }
    out
}

pub(crate) fn rand_elem<'a, T>(_rng: &mut dyn RngCore, _list: &'a Vec<T>) -> Option<&'a T> {
    if _list.is_empty() {
        return None;
    }
    let choice = _list.len().rands(_rng);
    let val = &_list[choice];
    Some(val)
}

pub(crate) fn rand_elem_mut<'a, T>(
    _rng: &mut dyn RngCore,
    _list: &'a mut Vec<T>,
) -> Option<&'a mut T> {
    if _list.is_empty() {
        return None;
    }
    let choice = _list.len().rands(_rng);
    let val = &mut _list[choice];
    Some(val)
}

pub(crate) fn mutate_num(_rng: &mut dyn RngCore, _num: i256) -> i256 {
    let choice = 12_usize.rands(_rng);
    let nums = interesting_numbers();
    match choice {
        0 => I256::from(_num).overflowing_add(I256::from(1)).0,
        1 => I256::from(_num).overflowing_sub(I256::from(1)).0,
        2 => I256::from(0),
        3 => I256::from(1),
        4 => *rand_elem(_rng, &nums).unwrap_or(&I256::from(0)),
        5 => *rand_elem(_rng, &nums).unwrap_or(&I256::from(0)),
        6 => *rand_elem(_rng, &nums).unwrap_or(&I256::from(0)),
        7 => {
            rand_elem(_rng, &nums)
                .unwrap_or(&I256::from(0))
                .rands(_rng)
                .overflowing_add(_num)
                .0
        }
        8 => {
            rand_elem(_rng, &nums)
                .unwrap_or(&I256::from(0))
                .rands(_rng)
                .overflowing_sub(_num)
                .0
        }
        9 => (_num * 2).rands(_rng).overflowing_sub(_num).0,
        _ => {
            let mut n = _rng.gen_range(1..129);
            n = n.rand_log(_rng);
            let s = 3.rands(_rng);
            let val = match s {
                0 => _num - n,
                _ => _num + n,
            };
            I256::from(val)
        }
    }
}

pub(crate) trait PriorityList {
    fn priority(&self) -> usize;
}

pub(crate) fn choose_priority<'a, T: PriorityList + std::fmt::Debug>(
    v: &'a mut Vec<T>,
    init: usize,
) -> Option<&'a mut T> {
    let len = v.len();
    let mut iter = v.iter_mut();
    let mut n: isize = init as isize;
    while let Some(next) = iter.next() {
        if n < next.priority() as isize {
            return Some(next);
        }
        if len == 1 {
            return Some(next);
        } else {
            n -= next.priority() as isize;
        }
    }
    None
}

pub(crate) fn rand_occurs(_rng: &mut dyn RngCore, prob: f64) -> bool {
    if prob.fract() == 0.0 {
        return false;
    }
    let f = Fraction::from(prob);
    let nom = *f.numer().unwrap();
    let denom = *f.denom().unwrap();
    let n = _rng.gen_range(0..denom);
    if nom == 1 {
        return n == 0;
    } else {
        return n < nom;
    }
}

pub(crate) fn _debug_escaped(input: &Vec<Vec<u8>>) {
    //let mut total_len = 0;
    for i in input {
        //total_len = total_len + i.len();
        let x = String::from_utf8(
            i.iter()
                .flat_map(|b| std::ascii::escape_default(*b))
                .collect::<Vec<u8>>(),
        )
        .unwrap();
        debug!("{}", x);
    }
}

pub fn get_files(_files: Vec<String>) -> Result<Vec<String>, GlobError<'static>> {
    let mut all_paths: Vec<String> = vec![];
    for f in _files {
        debug!("{}", f);
        let is_ip = Regex::new(r"([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+):([0-9]+)").unwrap();
        if is_ip.is_match(&f) {
            debug!("is address");
            all_paths.push(f);
        } else {
            let path = Path::new(&f);
            let (parent, filepattern) = match (path.is_dir(), path.is_file()) {
                (true, false) => (Some(path), "*".to_string()),
                (false, true) => (
                    path.parent(),
                    path.file_name().unwrap().to_str().unwrap().to_string(),
                ),
                _ => {
                    if path.is_relative() {
                        (
                            path.parent(),
                            path.file_name().unwrap().to_str().unwrap().to_string(),
                        )
                    } else {
                        (path.parent(), f.to_string())
                    }
                }
            };
            let parent = parent.unwrap().canonicalize().ok();
            if let Some(g) = Glob::new(&filepattern).ok() {
                let dir_path = parent.unwrap_or(".".into());
                for entry in g.walk(dir_path, 1) {
                    if let Some(e) = entry.ok() {
                        if e.file_type().is_file() {
                            let filepath = e.path().to_string_lossy().to_string();
                            debug!("Adding file {:#?}", filepath);
                            all_paths.push(filepath);
                        }
                    }
                }
            }
        }
    }
    Ok(all_paths)
}

pub(crate) fn _debug_type_of<T>(_: &T) {
    debug!("{}", std::any::type_name::<T>())
}

// Errors
#[derive(Debug, Clone)]
pub struct NoneString;
impl std::fmt::Display for NoneString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "String is None")
    }
}
impl std::error::Error for NoneString {}

#[derive(Debug, Clone)]
pub struct NoWrite;
impl std::fmt::Display for NoWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Does not impliment Write")
    }
}
impl std::error::Error for NoWrite {}

#[derive(Debug, Clone)]
pub struct BadInput;
impl std::fmt::Display for BadInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "String input could not be parsed")
    }
}
impl std::error::Error for BadInput {}

#[derive(Debug, Clone)]
pub struct NoStdin;
impl std::fmt::Display for NoStdin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Stdin is not available")
    }
}
impl std::error::Error for NoStdin {}

pub(crate) fn is_binarish(_data: Option<&Vec<u8>>) -> bool {
    let mut p = 0;
    if let Some(data) = _data {
        for b in data {
            if p == 8 {
                return false;
            }
            if *b == 0 {
                return true;
            }
            if (*b & 128) == 0 {
                p += 1;
            } else {
                return true;
            }
        }
    }
    false
}
