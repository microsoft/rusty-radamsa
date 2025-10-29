use rand::{Rng, RngCore};

use crate::shared::*;

/// connect prefix of al somewhere to bl, and make sure that (list-fuse l l) != l
pub fn fuse<T: Clone + PartialEq + std::fmt::Debug + std::hash::Hash + Eq + Ord>(
    rng: &mut dyn RngCore,
    lista: &Vec<T>,
    listb: &Vec<T>,
) -> Vec<T> {
    //find-jump-points
    if lista.is_empty() || listb.is_empty() {
        return lista.clone();
    }
    let (from, mut to) = find_jump_points(rng, lista, listb);
    // split and fold
    if let Some(prefix) = lista.strip_suffix(from.as_slice()) {
        let mut new_data = prefix.to_vec();
        new_data.append(&mut to);
        return new_data;
    }
    lista.clone()
}

fn alernate_suffixes<'a, T: Clone>(
    rng: &mut dyn RngCore,
    lista: &'a [T],
) -> (Vec<&'a [T]>, Vec<&'a [T]>) {
    let mut new_lista: Vec<&[T]> = Vec::new();
    let mut new_listb: Vec<&[T]> = Vec::new();
    let mut sub_lista: &[T] = &[];
    let mut sub_listb: &[T] = &[];
    for (i, _val) in lista.iter().enumerate() {
        let d: usize = rng.r#gen();
        if d & 1 == 1 {
            sub_lista = &lista[i..];
            if !sub_listb.is_empty() {
                new_listb.push(sub_listb);
                //sub_listb = Vec::new();
            }
        } else {
            sub_listb = &lista[i..];
            if !sub_lista.is_empty() {
                new_lista.push(sub_lista);
                //sub_lista = Vec::new();
            }
        }
    }

    (new_lista, new_listb)
}

/// avoid usually jumping into the same place (ft mutation, small samples, bad luck).
/// if the inputs happen to be equal by alternating possible jump and land positions.
fn initial_suffixes<'a, T: Clone + PartialEq>(
    rng: &mut dyn RngCore,
    lista: &'a Vec<T>,
    listb: &'a Vec<T>,
) -> (Vec<&'a [T]>, Vec<&'a [T]>) {
    // collect various suffixes
    if *lista == *listb {
        return alernate_suffixes(rng, lista);
    }
    (suffixes(rng, lista), suffixes(rng, listb))
}

fn suffixes<'a, T: Clone + PartialEq>(_rng: &mut dyn RngCore, list: &'a [T]) -> Vec<&'a [T]> {
    let mut new_list: Vec<&[T]> = Vec::new();
    for (i, _val) in list.iter().enumerate() {
        let sub_list: &[T] = &list[i..];
        new_list.push(sub_list);
    }
    new_list
}

fn any_position_pair<'a, T: Clone>(
    rng: &mut dyn RngCore,
    lista: &'a mut [T],
    listb: &'a mut [T],
) -> Option<(&'a mut T, &'a mut T)> {
    match (rand_elem_mut(rng, lista), rand_elem_mut(rng, listb)) {
        (Some(from), Some(to)) => Some((from, to)),
        _ => None,
    }
}

const SEARCH_FUEL: isize = 100_000;
const SEARCH_STOP_IP: usize = 8;

#[allow(suspicious_double_ref_op)]
#[allow(clippy::ptr_arg)]
fn split_prefixes<'a, T: Clone + PartialEq + std::fmt::Debug + std::hash::Hash + Eq + Ord>(
    prefixes: &Vec<&'a [T]>,
    suffixes: &Vec<&'a [T]>,
) -> (Vec<&'a [T]>, Vec<&'a [T]>) {
    let mut new_prefixes: Vec<&[T]> = Vec::new();
    let mut suffixes = suffixes.clone();
    let mut char_suffix = std::collections::BTreeSet::new();
    let mut hash_suffix: std::collections::BTreeSet<&[T]> = std::collections::BTreeSet::new();
    // assuming _prefixes is sorted by length
    for prefix in prefixes {
        if let Some(key) = prefix.first() && char_suffix.insert(key) {
            let len = prefix.len() - 1;
            new_prefixes.push(prefix.clone());
            suffixes.retain(|x| {
                if x.len() < len {
                    hash_suffix.insert(x.clone());
                    false
                } else {
                    true
                }
            });
        }
    }
    let new_suffixes: Vec<&[T]> = hash_suffix.into_iter().collect();
    (new_prefixes, new_suffixes)
}

#[allow(clippy::cast_possible_wrap)]
fn find_jump_points<T: Clone + PartialEq + std::fmt::Debug + std::hash::Hash + Eq + Ord>(
    rng: &mut dyn RngCore,
    lista: &Vec<T>,
    listb: &Vec<T>,
) -> (Vec<T>, Vec<T>) {
    let mut fuel = SEARCH_FUEL;
    let (mut la, mut lb) = initial_suffixes(rng, lista, listb);
    if la.is_empty() || lb.is_empty() {
        return (lista.clone(), listb.clone());
    }
    loop {
        if fuel < 0 {
            return match any_position_pair(rng, &mut la, &mut lb) {
                Some((from, to)) => (from.to_vec(), to.to_vec()),
                None => (lista.clone(), listb.clone()),
            };
        }

        let x = SEARCH_STOP_IP.rands(rng);
        if x == 0 {
            return match any_position_pair(rng, &mut la, &mut lb) {
                Some((from, to)) => (from.to_vec(), to.to_vec()),
                None => (lista.clone(), listb.clone()),
            };
        }

        let (nodea, nodeb) = split_prefixes(&la, &lb);
        if nodea.is_empty() || nodeb.is_empty() {
            return match any_position_pair(rng, &mut la, &mut lb) {
                Some((from, to)) => (from.to_vec(), to.to_vec()),
                None => (lista.clone(), listb.clone()),
            };
        }

        la = nodea;
        lb = nodeb;
        fuel -= (la.len() + lb.len()) as isize;
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    use super::*;

    #[test]
    fn test_alternating() {
        let data: Vec<u8> = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\n".to_vec();
        let mut rng = ChaCha20Rng::seed_from_u64(3);
        let new_data = fuse(&mut rng, &data, &data);
        assert_eq!(
            new_data,
            vec![
                65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 69, 70, 71, 72, 73, 74, 75,
                76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 10
            ]
        );
    }

    #[test]
    fn test_empty_fuse() {
        // Ensure no crashes
        let data: Vec<u8> = vec![];
        let mut rng = ChaCha20Rng::seed_from_u64(3);
        let _new_data = fuse(&mut rng, &data, &data);
    }
}
