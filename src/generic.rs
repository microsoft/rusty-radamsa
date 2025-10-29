#[cfg(test)]
use std::println as debug;

#[cfg(not(test))]
use log::debug;
use rand::{seq::SliceRandom, RngCore};

use crate::shared::*;

/// delete a sequence of things
pub fn list_del_seq<T: Clone>(rng: &mut dyn RngCore, data: Vec<T>) -> Vec<T> {
    if data.len() < 2 {
        data
    } else {
        let s = safe_gen_range(rng, 0, data.len() - 1);
        let e = safe_gen_range(rng, s + 1, data.len());
        let mut new_data: Vec<T> = Vec::new();
        new_data.extend(data[..s].to_vec());
        new_data.extend(data[e..].to_vec());
        new_data
    }
}

/// delete a random element
pub fn list_del<T: Clone>(rng: &mut dyn RngCore, data: Vec<T>) -> Vec<T> {
    if data.len() < 2 {
        data
    } else {
        let pos = safe_gen_range(rng, 0, data.len());
        let mut new_data: Vec<T> = data;
        new_data.remove(pos);
        new_data
    }
}

/// duplicate a random element
pub fn list_dup<T: Clone>(rng: &mut dyn RngCore, data: Vec<T>) -> Vec<T> {
    if data.is_empty() {
        data
    } else if data.len() < 2 {
        let mut new_data: Vec<T> = data;
        let item = new_data[0].clone();
        new_data.push(item);
        new_data
    } else {
        let pos = safe_gen_range(rng, 0, data.len() - 1);
        let mut new_data: Vec<T> = data;
        let new_item = new_data[pos].clone();
        new_data.insert(pos + 1, new_item);
        new_data
    }
}

/// clone a value to another position
pub fn list_clone<T: Clone>(rng: &mut dyn RngCore, data: Vec<T>) -> Vec<T> {
    if data.is_empty() {
        data
    } else if data.len() < 2 {
        let mut new_data: Vec<T> = data;
        let item = new_data[0].clone();
        new_data.push(item);
        new_data
    } else {
        let pos = safe_gen_range(rng, 0, data.len());
        let new_pos = safe_gen_range(rng, 0, data.len());
        let mut new_data: Vec<T> = data;
        let new_item = new_data[pos].clone();
        new_data.insert(new_pos, new_item);
        new_data
    }
}

/// swap two adjecent values
pub fn list_swap<T: Clone>(rng: &mut dyn RngCore, data: Vec<T>) -> Vec<T> {
    if data.len() < 2 {
        data
    } else {
        let pos = safe_gen_range(rng, 0, data.len() - 1);
        let adjecent = pos + 1;
        let mut new_data: Vec<T> = data;
        new_data.swap(pos, adjecent);
        new_data
    }
}

/// permute values
pub fn list_perm<T: Clone>(rng: &mut dyn RngCore, data: Vec<T>) -> Vec<T> {
    debug!("list_perm");
    if data.len() < 3 {
        data
    } else {
        let min_range = data.len() - 3;
        let from = match min_range {
            0 => 0,
            _ => safe_gen_range(rng, 0, min_range),
        };
        let max_range = data.len() - from;
        let a = match max_range {
            0 => safe_gen_range(rng, from, data.len()),
            _ => safe_gen_range(rng, from, max_range),
        };
        let b = 10_usize.rand_log(rng);
        let n = std::cmp::max(2, std::cmp::min(a, b));
        let mut new_data: Vec<T> = data;
        new_data[from..from + n].shuffle(rng);
        new_data
    }
}

/// repeat an element
pub fn list_repeat<T: Clone>(rng: &mut dyn RngCore, data: Vec<T>) -> Vec<T> {
    if data.is_empty() {
        return data;
    }
    let pos = safe_gen_range(rng, 0, data.len());
    let mut n = 10_usize.rand_log(rng);
    n = std::cmp::max(2, n);
    let mut new_data: Vec<T> = data[..pos].to_vec();
    for _i in 0..n {
        let item = data[pos].clone();
        new_data.push(item);
    }
    new_data.extend(data[pos..].to_vec());
    new_data
}

/// insert a line from elsewhere
pub fn list_ins<T: Clone>(rng: &mut dyn RngCore, data: Vec<T>) -> Vec<T> {
    if data.is_empty() {
        data
    } else if data.len() < 2 {
        let mut new_data: Vec<T> = data;
        let item = new_data[0].clone();
        new_data.push(item);
        new_data
    } else {
        let pos = safe_gen_range(rng, 0, data.len());
        let new_pos = safe_gen_range(rng, 0, data.len());
        let mut new_data: Vec<T> = data;
        let new_item = new_data[pos].clone();
        new_data.insert(new_pos, new_item);
        new_data
    }
}

/// clone a value to another position
pub fn list_replace<T: Clone>(rng: &mut dyn RngCore, data: Vec<T>) -> Vec<T> {
    if data.len() < 2 {
        data
    } else {
        let pos = safe_gen_range(rng, 0, data.len());
        let new_pos = safe_gen_range(rng, 0, data.len());
        let mut new_data: Vec<T> = data;
        let new_item = new_data[pos].clone();
        new_data.push(new_item);
        new_data.swap_remove(new_pos);
        new_data
    }
}

/// connect prefix of al somewhere to bl, and make sure that (list-fuse l l) != l
pub fn list_fuse<T: Clone + PartialEq + std::fmt::Debug + std::hash::Hash + Eq + Ord>(
    rng: &mut dyn RngCore,
    lista: &Vec<T>,
    listb: &Vec<T>,
) -> Vec<T> {
    crate::fuse::fuse(rng, lista, listb)
}
