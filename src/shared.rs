//! Utility functions for the crate.
//!
use std::path::Path;
use std::time::SystemTime;

use ethnum::*;
use fraction::Fraction;
use log::*;
use rand::{Rng, RngCore};
use regex::Regex;
use wax::{Glob, GlobError};

pub const AVG_BLOCK_SIZE: usize = 2048;
pub const MIN_BLOCK_SIZE: usize = 256;
pub const INITIAL_IP: usize = 24;
pub const MAX_BLOCK_SIZE: usize = 2 * AVG_BLOCK_SIZE;
pub const REMUTATE_PROBABILITY: f64 = 0.8; // 4/5
pub const MAX_CHECKSUM_RETRY: usize = 10000;
pub const MAX_UDP_PACKET_SIZE: usize = 65507;
pub const SILLY_STRINGS: [&str; 2] = ["cmd.exe", "/C"];

#[macro_export]
macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

pub(crate) fn time_seed() -> u64 {
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Duration since UNIX_EPOCH failed");
    d.as_secs()
}

pub trait Rands {
    #[must_use]
    fn rands(&self, _rng: &mut dyn RngCore) -> Self;
    #[must_use]
    fn rand_log(&self, _rng: &mut dyn RngCore) -> Self;
}

pub(crate) fn safe_gen_range(rng: &mut dyn RngCore, low: usize, high: usize) -> usize {
    if high == 0 {
        return high;
    }
    if low >= high {
        return high;
    }
    rng.gen_range(low..high)
}

#[allow(clippy::cast_possible_truncation)]
impl Rands for usize {
    fn rands(&self, rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        rng.gen_range(0..*self)
    }

    fn rand_log(&self, rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_usize.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(rng);
            return val | hi;
        }
        0
    }
}

#[allow(clippy::cast_possible_truncation)]
impl Rands for u64 {
    fn rands(&self, rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        rng.gen_range(0..*self)
    }

    fn rand_log(&self, rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_usize.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(rng);
            return (val | hi) as Self;
        }
        0
    }
}

#[allow(clippy::cast_possible_truncation)]
impl Rands for u128 {
    fn rands(&self, rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        rng.gen_range(0..*self)
    }

    fn rand_log(&self, rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_usize.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(rng);
            return (val | hi) as Self;
        }
        0
    }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
impl Rands for isize {
    fn rands(&self, rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        rng.gen_range(-*self..*self)
    }

    fn rand_log(&self, rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_isize.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(rng);
            return val | hi;
        }
        0
    }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
impl Rands for i128 {
    fn rands(&self, rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        rng.gen_range(-*self..*self)
    }

    fn rand_log(&self, rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_i128.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(rng);
            return val | hi;
        }
        0
    }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
impl Rands for i32 {
    fn rands(&self, rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        rng.gen_range(-*self..*self)
    }

    fn rand_log(&self, rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_i32.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(rng);
            return val | hi;
        }
        0
    }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
impl Rands for i64 {
    fn rands(&self, rng: &mut dyn RngCore) -> Self {
        if *self == 0 {
            return 0;
        }
        rng.gen_range(-*self..*self)
    }

    fn rand_log(&self, rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = rng.gen_range(0..*self);
            if n == 0 {
                return 0;
            }
            let hi = 1_i64.overflowing_shl(n as u32 - 1).0;
            let val = hi.rands(rng);
            return val | hi;
        }
        0
    }
}

impl Rands for i256 {
    fn rands(&self, rng: &mut dyn RngCore) -> Self {
        if *self == Self::from(0) {
            return Self::from(0);
        }
        if (self.as_i128().overflowing_abs().0 == Self::from(0)
            && self.as_i128().overflowing_neg().0 == Self::from(0))
            || self.as_i128().overflowing_abs().0 == self.as_i128().overflowing_neg().0
        {
            return Self::from(0);
        }
        Self::from(
            rng.gen_range(self.as_i128().overflowing_neg().0..self.as_i128().overflowing_abs().0),
        )
    }

    fn rand_log(&self, rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = self.rands(rng);
            if n == 0 {
                return Self::from(0);
            }
            let hi = Self::from(1).overflowing_shl(n.as_u32() - 1).0;
            let val = hi.rands(rng);
            return val | hi;
        }
        Self::from(0)
    }
}

impl Rands for u256 {
    fn rands(&self, rng: &mut dyn RngCore) -> Self {
        if *self == Self::from(0_u32) {
            return Self::from(0_u32);
        }
        Self::from(rng.gen_range(0..self.as_u128()))
    }

    fn rand_log(&self, rng: &mut dyn RngCore) -> Self {
        if *self != 0 {
            let n = self.rands(rng);
            if n == 0 {
                return Self::from(0_u32);
            }
            let hi = Self::from(1_u32).overflowing_shl(n.as_u32() - 1).0;
            let val = hi.rands(rng);
            return val | hi;
        }
        Self::from(0_u32)
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

pub(crate) fn rand_elem<'a, T>(rng: &mut dyn RngCore, list: &'a [T]) -> Option<&'a T> {
    if list.is_empty() {
        return None;
    }
    let choice = list.len().rands(rng);
    let val = &list[choice];
    Some(val)
}

pub(crate) fn rand_elem_mut<'a, T>(rng: &mut dyn RngCore, list: &'a mut [T]) -> Option<&'a mut T> {
    if list.is_empty() {
        return None;
    }
    let choice = list.len().rands(rng);
    let val = &mut list[choice];
    Some(val)
}

pub(crate) fn mutate_num(rng: &mut dyn RngCore, num: i256) -> i256 {
    let choice = 12_usize.rands(rng);
    let nums = interesting_numbers();
    match choice {
        0 => num.overflowing_add(I256::from(1)).0,
        1 => num.overflowing_sub(I256::from(1)).0,
        2 => I256::from(0),
        3 => I256::from(1),
        4..=6 => rand_elem(rng, &nums)
            .copied()
            .unwrap_or_else(|| I256::from(0)),
        7 => {
            rand_elem(rng, &nums)
                .copied()
                .unwrap_or_else(|| I256::from(0))
                .rands(rng)
                .overflowing_add(num)
                .0
        }
        8 => {
            rand_elem(rng, &nums)
                .copied()
                .unwrap_or_else(|| I256::from(0))
                .rands(rng)
                .overflowing_sub(num)
                .0
        }
        9 => (num * 2).rands(rng).overflowing_sub(num).0,
        _ => {
            let mut n = rng.gen_range(1..129);
            n = n.rand_log(rng);
            let s = 3.rands(rng);
            match s {
                0 => num - n,
                _ => num + n,
            }
        }
    }
}

pub(crate) trait PriorityList {
    fn priority(&self) -> usize;
}

#[allow(clippy::cast_possible_wrap)]
pub(crate) fn choose_priority<T: PriorityList + std::fmt::Debug>(
    v: &mut [T],
    init: usize,
) -> Option<&mut T> {
    let len = v.len();
    let mut n: isize = init as isize;
    for next in v.iter_mut() {
        if n < next.priority() as isize {
            return Some(next);
        }
        if len == 1 {
            return Some(next);
        }
        n -= next.priority() as isize;
    }
    None
}

pub(crate) fn rand_occurs(rng: &mut dyn RngCore, prob: f64) -> bool {
    if prob.fract() == 0.0 {
        return false;
    }
    let f = Fraction::from(prob);
    let nom = *f.numer().unwrap();
    let denom = *f.denom().unwrap();
    let n = rng.gen_range(0..denom);
    if nom == 1 {
        n == 0
    } else {
        n < nom
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

pub fn get_files(files: Vec<String>) -> Result<Vec<String>, GlobError<'static>> {
    let mut all_paths: Vec<String> = vec![];
    let is_ip = Regex::new(r"([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+):([0-9]+)").unwrap();
    for f in files {
        debug!("{}", f);
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
            if let Ok(g) = Glob::new(&filepattern) {
                let dir_path = parent.unwrap_or_else(|| ".".into());
                for entry in g.walk(dir_path, 1).flatten() {
                    if entry.file_type().is_file() {
                        let filepath = entry.path().to_string_lossy().to_string();
                        debug!("Adding file {:#?}", filepath);
                        all_paths.push(filepath);
                    }
                }
            }
        }
    }
    Ok(all_paths)
}

pub(crate) fn _debug_type_of<T>(_: &T) {
    debug!("{}", std::any::type_name::<T>());
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

pub(crate) fn is_binarish(data: Option<&Vec<u8>>) -> bool {
    let mut p = 0;
    if let Some(data) = data {
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
