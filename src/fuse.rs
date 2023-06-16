use crate::shared::*;
use rand::{Rng, RngCore};

/// connect prefix of al somewhere to bl, and make sure that (list-fuse l l) != l
pub(crate) fn fuse<
    T: Clone + std::cmp::PartialEq + std::fmt::Debug + std::hash::Hash + std::cmp::Eq + std::cmp::Ord,
>(
    _rng: &mut dyn RngCore,
    _lista: &Vec<T>,
    _listb: &Vec<T>,
) -> Vec<T> {
    //find-jump-points
    if _lista.is_empty() || _listb.is_empty() {
        return _lista.clone();
    }
    let (from, mut to) = find_jump_points(_rng, &_lista, &_listb);
    // split and fold
    if let Some(prefix) = _lista.strip_suffix(from.as_slice()) {
        let mut new_data = prefix.to_vec();
        new_data.append(&mut to);
        return new_data;
    }
    _lista.clone()
}

fn alernate_suffixes<'a, T: Clone>(
    _rng: &mut dyn RngCore,
    _lista: &'a Vec<T>,
) -> (Vec<&'a [T]>, Vec<&'a [T]>) {
    let mut new_lista: Vec<&[T]> = Vec::new();
    let mut new_listb: Vec<&[T]> = Vec::new();
    let mut sub_lista: &[T] = &[];
    let mut sub_listb: &[T] = &[];
    for (i, _val) in _lista.iter().enumerate() {
        let d: usize = _rng.gen();
        if d & 1 == 1 {
            sub_lista = &_lista[i..];
            if !sub_listb.is_empty() {
                new_listb.push(sub_listb);
                //sub_listb = Vec::new();
            }
        } else {
            sub_listb = &_lista[i..];
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
fn initial_suffixes<'a, T: Clone + std::cmp::PartialEq>(
    _rng: &mut dyn RngCore,
    _lista: &'a Vec<T>,
    _listb: &'a Vec<T>,
) -> (Vec<&'a [T]>, Vec<&'a [T]>) {
    // collect various suffixes
    if *_lista == *_listb {
        return alernate_suffixes(_rng, _lista);
    }
    (suffixes(_rng, _lista), suffixes(_rng, _listb))
}

fn suffixes<'a, T: Clone + std::cmp::PartialEq>(
    _rng: &mut dyn RngCore,
    _list: &'a Vec<T>,
) -> Vec<&'a [T]> {
    let mut new_list: Vec<&[T]> = Vec::new();
    for (i, _val) in _list.iter().enumerate() {
        let mut sub_list: &[T] = &[];
        sub_list = &_list[i..];
        new_list.push(sub_list);
    }
    new_list
}

fn any_position_pair<'a, T: Clone>(
    _rng: &mut dyn RngCore,
    _lista: &'a mut Vec<T>,
    _listb: &'a mut Vec<T>,
) -> Option<(&'a mut T, &'a mut T)> {
    match (rand_elem_mut(_rng, _lista), rand_elem_mut(_rng, _listb)) {
        (Some(from), Some(to)) => Some((from, to)),
        _ => None,
    }
}

const SEARCH_FUEL: isize = 100000;
const SEARCH_STOP_IP: usize = 8;

fn split_prefixes<
    'a,
    T: Clone + std::cmp::PartialEq + std::fmt::Debug + std::hash::Hash + std::cmp::Eq + std::cmp::Ord,
>(
    _prefixes: &Vec<&'a [T]>,
    _suffixes: &Vec<&'a [T]>,
) -> (Vec<&'a [T]>, Vec<&'a [T]>) {
    let mut new_prefixes: Vec<&[T]> = Vec::new();
    let mut suffixes = _suffixes.clone();
    let mut char_suffix = std::collections::BTreeSet::new();
    let mut hash_suffix: std::collections::BTreeSet<&[T]> = std::collections::BTreeSet::new();
    // assuming _prefixes is sorted by length
    for prefix in _prefixes {
        if let Some(key) = prefix.first() {
            if char_suffix.get(key).is_none() {
                let len = prefix.len() - 1;
                new_prefixes.push(prefix.clone());
                char_suffix.insert(key);
                suffixes.retain(|x| {
                    if x.len() < len {
                        hash_suffix.insert(x.clone());
                        false
                    } else {
                        true
                    }
                });
                if suffixes.len() == 0 {
                    continue;
                }
            }
        }
    }
    let new_suffixes: Vec<&[T]> = hash_suffix.into_iter().collect();
    (new_prefixes, new_suffixes)
}

fn find_jump_points<
    T: Clone + std::cmp::PartialEq + std::fmt::Debug + std::hash::Hash + std::cmp::Eq + std::cmp::Ord,
>(
    _rng: &mut dyn RngCore,
    _lista: &Vec<T>,
    _listb: &Vec<T>,
) -> (Vec<T>, Vec<T>) {
    let mut fuel = SEARCH_FUEL;
    let (mut lista, mut listb) = initial_suffixes(_rng, _lista, _listb);
    if lista.is_empty() || listb.is_empty() {
        return (_lista.to_vec(), _listb.to_vec());
    }
    loop {
        if fuel < 0 {
            match any_position_pair(_rng, &mut lista, &mut listb) {
                Some((from, to)) => return (from.to_vec(), to.to_vec()),
                None => return (_lista.to_vec(), _listb.to_vec()),
            }
        } else {
            let x = SEARCH_STOP_IP.rands(_rng);
            if x == 0 {
                match any_position_pair(_rng, &mut lista, &mut listb) {
                    Some((from, to)) => return (from.to_vec(), to.to_vec()),
                    None => return (_lista.to_vec(), _listb.to_vec()),
                }
            } else {
                let (nodea, nodeb) = split_prefixes(&lista, &listb);
                if nodea.is_empty() || nodeb.is_empty() {
                    match any_position_pair(_rng, &mut lista, &mut listb) {
                        Some((from, to)) => return (from.to_vec(), to.to_vec()),
                        None => return (_lista.to_vec(), _listb.to_vec()),
                    }
                } else {
                    lista = nodea;
                    listb = nodeb;
                    fuel -= (lista.len() + listb.len()) as isize;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_alternating() {
        let data: Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\n".as_bytes().to_vec();
        let mut rng = ChaCha20Rng::seed_from_u64(3);
        let new_data = fuse(&mut rng, &data, &data);
        assert_eq!(
            new_data,
            vec![
                65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 69, 70, 71, 72, 73, 74, 75,
                76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 10
            ]
        )
    }

    #[test]
    fn test_empty_fuse() {
        // Ensure no crashes
        let data: Vec<u8> = vec![];
        let mut rng = ChaCha20Rng::seed_from_u64(3);
        let _new_data = fuse(&mut rng, &data, &data);
    }
}
