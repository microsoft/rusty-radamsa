use crate::shared::*;
use rand::{seq::SliceRandom, RngCore};

#[cfg(not(test))]
use log::debug;

#[cfg(test)]
use std::println as debug;

/// delete a sequence of things
pub fn list_del_seq<T: std::clone::Clone>(_rng: &mut dyn RngCore, _data: Vec<T>) -> Vec<T> {
    if _data.len() < 2 {
        _data
    } else {
        let s = safe_gen_range(_rng, 0, _data.len() - 1);
        let e = safe_gen_range(_rng, s + 1, _data.len());
        let mut new_data: Vec<T> = Vec::new();
        new_data.extend(_data[..s].to_vec());
        new_data.extend(_data[e..].to_vec());
        new_data
    }
}

/// delete a random element
pub fn list_del<T: Clone>(_rng: &mut dyn RngCore, _data: Vec<T>) -> Vec<T> {
    if _data.len() < 2 {
        _data
    } else {
        let pos = safe_gen_range(_rng, 0, _data.len());
        let mut new_data: Vec<T> = _data;
        new_data.remove(pos);
        new_data
    }
}

/// duplicate a random element
pub fn list_dup<T: Clone>(_rng: &mut dyn RngCore, _data: Vec<T>) -> Vec<T> {
    if _data.is_empty() {
        _data
    } else if _data.len() < 2 {
        let mut new_data: Vec<T> = _data;
        let item = new_data[0].clone();
        new_data.push(item);
        new_data
    } else {
        let pos = safe_gen_range(_rng, 0, _data.len() - 1);
        let mut new_data: Vec<T> = _data;
        let new_item = new_data[pos].clone();
        new_data.insert(pos + 1, new_item);
        new_data
    }
}

/// clone a value to another position
pub fn list_clone<T: Clone>(_rng: &mut dyn RngCore, _data: Vec<T>) -> Vec<T> {
    if _data.is_empty() {
        _data
    } else if _data.len() < 2 {
        let mut new_data: Vec<T> = _data;
        let item = new_data[0].clone();
        new_data.push(item);
        new_data
    } else {
        let pos = safe_gen_range(_rng, 0, _data.len());
        let new_pos = safe_gen_range(_rng, 0, _data.len());
        let mut new_data: Vec<T> = _data;
        let new_item = new_data[pos].clone();
        new_data.insert(new_pos, new_item);
        new_data
    }
}

/// swap two adjecent values
pub fn list_swap<T: Clone>(_rng: &mut dyn RngCore, _data: Vec<T>) -> Vec<T> {
    if _data.len() < 2 {
        _data
    } else {
        let pos = safe_gen_range(_rng, 0, _data.len() - 1);
        let adjecent = pos + 1;
        let mut new_data: Vec<T> = _data;
        new_data.swap(pos, adjecent);
        new_data
    }
}

/// permute values
pub fn list_perm<T: Clone>(_rng: &mut dyn RngCore, _data: Vec<T>) -> Vec<T> {
    debug!("list_perm");
    if _data.len() < 3 {
        _data.to_vec()
    } else {
        let min_range = _data.len() - 3;
        let from = match min_range {
            0 => 0,
            _ => safe_gen_range(_rng, 0, min_range),
        };
        let max_range = _data.len() - from;
        let a = match max_range {
            0 => safe_gen_range(_rng, from, _data.len()),
            _ => safe_gen_range(_rng, from, max_range),
        };
        let b = 10_usize.rand_log(_rng);
        let n = std::cmp::max(2, std::cmp::min(a, b));
        let mut new_data: Vec<T> = _data;
        new_data[from..from + n].shuffle(_rng);
        new_data
    }
}

/// repeat an element
pub fn list_repeat<T: Clone>(_rng: &mut dyn RngCore, _data: Vec<T>) -> Vec<T> {
    if _data.is_empty() {
        return _data;
    }
    let pos = safe_gen_range(_rng, 0, _data.len());
    let mut n = 10_usize.rand_log(_rng);
    n = std::cmp::max(2, n);
    let mut new_data: Vec<T> = _data[..pos].to_vec();
    for _i in 0..n {
        let item = _data[pos].clone();
        new_data.push(item);
    }
    new_data.extend(_data[pos..].to_vec());
    new_data
}

/// insert a line from elsewhere
pub fn list_ins<T: Clone>(_rng: &mut dyn RngCore, _data: Vec<T>) -> Vec<T> {
    if _data.is_empty() {
        _data
    } else if _data.len() < 2 {
        let mut new_data: Vec<T> = _data;
        let item = new_data[0].clone();
        new_data.push(item);
        new_data
    } else {
        let pos = safe_gen_range(_rng, 0, _data.len());
        let new_pos = safe_gen_range(_rng, 0, _data.len());
        let mut new_data: Vec<T> = _data;
        let new_item = new_data[pos].clone();
        new_data.insert(new_pos, new_item);
        new_data
    }
}

/// clone a value to another position
pub fn list_replace<T: Clone>(_rng: &mut dyn RngCore, _data: Vec<T>) -> Vec<T> {
    if _data.len() < 2 {
        _data
    } else {
        let pos = safe_gen_range(_rng, 0, _data.len());
        let new_pos = safe_gen_range(_rng, 0, _data.len());
        let mut new_data: Vec<T> = _data;
        let new_item = new_data[pos].clone();
        new_data.push(new_item);
        new_data.swap_remove(new_pos);
        new_data
    }
}

/// connect prefix of al somewhere to bl, and make sure that (list-fuse l l) != l
pub fn list_fuse<
    T: Clone + std::cmp::PartialEq + std::fmt::Debug + std::hash::Hash + std::cmp::Eq + std::cmp::Ord,
>(
    _rng: &mut dyn RngCore,
    _lista: &Vec<T>,
    _listb: &Vec<T>,
) -> Vec<T> {
    crate::fuse::fuse(_rng, _lista, _listb)
}
