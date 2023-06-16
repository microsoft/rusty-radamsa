/// Note that these functions are not one-to-one mappings of radamsa
use crate::shared::*;
use crate::RngCore;
use rand::prelude::SliceRandom;
use snowflake::ProcessUniqueId;

#[cfg(not(test))]
use log::debug;

#[cfg(test)]
use std::println as debug;

pub const MAX_LEVELS: usize = 256;

pub const USUAL_DELIMS: [(u8, u8); 6] =
    [(40, 41), (91, 93), (60, 62), (123, 125), (34, 34), (39, 39)];

const COMMA: u8 = 44;

#[derive(Debug, Clone, Default)]
struct Node {
    pub id: ProcessUniqueId,
    pub level: usize,
    pub delim: (u8, u8),
    pub start_index: usize,
    pub end_index: usize,
    pub parent_id: Option<ProcessUniqueId>,
    pub children: Vec<Node>,
    pub needs_separator: bool,
}

impl Node {
    fn new(_start: usize, _delim: (u8, u8)) -> Self {
        Node {
            id: ProcessUniqueId::new(),
            level: 0,
            delim: _delim,
            start_index: _start,
            end_index: _start,
            parent_id: None,
            children: Vec::new(),
            needs_separator: false,
        }
    }

    fn add_child(&mut self, child: Node) {
        let mut c = child;
        if self.level < MAX_LEVELS {
            c.parent_id = Some(self.id);
            self.children.push(c);
        } else {
            debug!("Max levels reached");
        }
    }
    fn set_end_index(&mut self, end: usize) {
        self.end_index = end;
    }
    fn get_mut<'a>(&'a mut self, node_id: ProcessUniqueId) -> Option<&'a mut Node> {
        for child in &mut self.children {
            if child.id == node_id {
                return Some(child);
            } else {
                let target = child.get_mut(node_id);
                if target.is_some() {
                    return target;
                }
            }
        }

        None
    }

    fn copy(&self) -> Node {
        Node {
            id: self.id,
            level: self.level,
            delim: self.delim,
            start_index: self.start_index,
            end_index: self.end_index,
            parent_id: self.parent_id,
            children: self.children.clone(),
            needs_separator: self.needs_separator,
        }
    }
}

fn check_delim_open(byte: &u8) -> Option<(u8, u8)> {
    for delim in USUAL_DELIMS {
        if delim.0 == *byte {
            return Some(delim);
        }
    }
    None
}

fn check_delim_close(byte: &u8) -> Option<(u8, u8)> {
    for delim in USUAL_DELIMS {
        if delim.1 == *byte {
            return Some(delim);
        }
    }
    None
}

fn check_node(node: &Node, _delim: Option<(u8, u8)>, index: usize) -> bool {
    if let Some(delim) = _delim {
        if node.delim.0 == delim.0 {
            if index != node.start_index {
                return true;
            }
        }
    }
    false
}

fn build_binary_tree(bytes: &Vec<u8>) -> Option<Node> {
    // use a stack to keep track of the nodes
    let mut stack: Vec<Node> = Vec::new();
    let mut root_node = Node::new(0, (0, 0));
    root_node.set_end_index(bytes.len());

    for (index, byte) in bytes.iter().enumerate() {
        let close_delim = check_delim_close(byte);
        if close_delim.is_some()
            && stack
                .last()
                .map_or(false, |n| check_node(n, close_delim, index))
        {
            let mut node = stack.pop().expect("invalid parentheses sequence");
            node.set_end_index(index + 1);
            node.delim = close_delim.unwrap();
            if let Some(parent) = stack.last_mut() {
                node.level = parent.level + 1;
                parent.add_child(node);
            } else {
                root_node.add_child(node);
            }
        } else if let Some(delim) = check_delim_open(byte) {
            // push a new node onto the stack
            let node = Node::new(index, (delim.0, 0));
            stack.push(node);
        } else {
            let mut node = Node::new(index, (0, 0));
            node.set_end_index(index + 1);
            if let Some(parent) = stack.last_mut() {
                node.level = parent.level + 1;
                parent.add_child(node);
            } else {
                root_node.add_child(node);
            }
        }
    }
    // add whats left
    root_node.children.append(&mut stack);
    Some(root_node)
}

fn partial_parse(_data: &Vec<u8>) -> Option<Node> {
    if is_binarish(Some(_data)) {
        return None;
    } else {
        return build_binary_tree(_data);
    }
}
fn sublist(_node: &Node) -> Vec<ProcessUniqueId> {
    let mut id_list: Vec<ProcessUniqueId> = Vec::new();
    // ignore root node and empty pairs
    if _node.start_index != _node.end_index && _node.delim != (0, 0) {
        id_list.push(_node.id);
    }
    for child in &_node.children {
        let mut new_ids = sublist(&child);
        id_list.append(&mut new_ids);
    }
    id_list
}

pub enum TreeMutate {
    TreeDup,         // tr2 - duplicate adjacently
    TreeDel,         // td - remove node
    TreeStutter,     // tr - repeat node as nested children
    TreeSwapReplace, // ts1 - copy 1 and replace the other
    TreeSwapPair,    // ts2 - swap 2 nodes
}

fn pick_sublist<'a>(_rng: &mut dyn RngCore, _tree: &'a mut Node) -> Option<&'a mut Node> {
    let mut id_list: Vec<ProcessUniqueId> = Vec::new();
    let mut new_ids = sublist(_tree);
    id_list.append(&mut new_ids);

    if id_list.is_empty() {
        return None;
    }
    let node_id = rand_elem(_rng, &id_list)?;
    return _tree.get_mut(*node_id);
}

fn _print_binary_tree(_node: &Node, level: usize) {
    if _node.start_index != _node.end_index && _node.delim != (0, 0) {
        debug!(
            "{} {} {} {} {}",
            " ".repeat(level),
            _node.delim.0 as char,
            _node.level,
            _node.start_index,
            _node.needs_separator
        );
    }
    let new_level = level + 1;
    for child in &_node.children {
        _print_binary_tree(child, new_level);
    }
    if _node.end_index != _node.start_index && _node.delim != (0, 0) {
        debug!(
            "{} {} {} {}",
            " ".repeat(level),
            _node.delim.1 as char,
            _node.level,
            _node.end_index
        );
    }
}

// check for comma separator
fn check_separator(_start_index: usize, _data: &Vec<u8>) -> bool {
    let prev_index = _start_index - 1;
    if let Some(prev_byte) = _data.get(prev_index) {
        if *prev_byte == COMMA {
            return true;
        }
    }
    false
}

fn tree_to_vec(_tree: &Node, _data: &Vec<u8>) -> Option<Vec<u8>> {
    let mut new_data = Vec::new();
    let mut og_data = _data[_tree.start_index.._tree.end_index].to_vec();
    if _tree.children.is_empty() {
        new_data.append(&mut og_data);
    } else {
        if _tree.needs_separator {
            new_data.push(COMMA);
        }
        if _tree.delim != (0, 0) && _tree.delim.0 != 0 {
            new_data.push(_tree.delim.0);
        }
        for child in &_tree.children {
            if let Some(mut child_data) = tree_to_vec(child, _data) {
                new_data.append(&mut child_data);
            }
        }
        if _tree.delim != (0, 0) && _tree.delim.1 != 0 {
            new_data.push(_tree.delim.1);
        }
    }
    Some(new_data)
}

fn repeat_path(_parent_node: &mut Node, _child_index: usize, _n_rep: usize) {
    let parent_copy = _parent_node.copy();
    if 0 < _n_rep {
        let node = _parent_node.children.get_mut(_child_index).unwrap();
        *node = parent_copy;
        repeat_path(node, _child_index, _n_rep - 1);
    }
}

pub(crate) fn sed_tree_op(
    _rng: &mut dyn RngCore,
    _data: &Vec<u8>,
    _mutate_type: TreeMutate,
) -> Option<Vec<u8>> {
    // parse data to tree if not binaryish
    let mut tree = partial_parse(_data)?;

    match _mutate_type {
        TreeMutate::TreeDup => {
            // add duplicate node to parent
            let mut node = pick_sublist(_rng, &mut tree)?.clone();
            let parent_id = node.parent_id?;
            let parent_node = tree.get_mut(parent_id)?;
            // get index of child
            let index = parent_node
                .children
                .iter()
                .position(|r| r.id == node.id)
                .unwrap();
            node.needs_separator = check_separator(node.start_index, _data);
            node.id = ProcessUniqueId::new();
            for child in &mut node.children {
                child.parent_id = Some(node.id);
            }
            parent_node.children.insert(index + 1, node);
        }
        TreeMutate::TreeDel => {
            let node = pick_sublist(_rng, &mut tree)?.clone();
            let parent_id = node.parent_id?;
            let parent_node = tree.get_mut(parent_id)?;
            // get index of child
            let index = parent_node
                .children
                .iter()
                .position(|r| r.id == node.id)
                .unwrap();
            parent_node.children.remove(index);
        }
        TreeMutate::TreeStutter => {
            let n_reps = 10.rand_log(_rng);
            let node = pick_sublist(_rng, &mut tree)?.clone();
            let parent_id = node.parent_id?;
            let mut parent_node = tree.get_mut(parent_id)?;
            let index = parent_node
                .children
                .iter()
                .position(|r| r.id == node.id)
                .unwrap();
            repeat_path(&mut parent_node, index, n_reps);
            debug!("n_reps: {}", n_reps);
        }
        TreeMutate::TreeSwapReplace => {
            let mut node_list = sublist(&mut tree);
            if node_list.len() < 2 {
                return None;
            }
            // permute
            node_list.shuffle(_rng);
            let toswap_id = node_list.first()?;
            // safe to unwrap here because the id exists.
            let toswap_node = tree.get_mut(*toswap_id)?.clone();
            let node = pick_sublist(_rng, &mut tree)?;
            *node = toswap_node;
        }
        TreeMutate::TreeSwapPair => {
            let mut node_list = sublist(&mut tree);
            if node_list.len() < 2 {
                return None;
            }
            // permute
            node_list.shuffle(_rng);
            let toswap_id = node_list.first()?;
            // safe to unwrap here because the id exists.
            let toswap_node = tree.get_mut(*toswap_id)?.clone();
            let node = pick_sublist(_rng, &mut tree)?;
            let old_node = node.clone();
            *node = toswap_node;
            let toswap_og = tree.get_mut(*toswap_id)?;
            *toswap_og = old_node;
        }
    }
    let new_data = tree_to_vec(&tree, _data)?;
    Some(new_data)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use print_bytes::println_lossy;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_tree_dup() {
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        // Broken json
        let expected =
            "{{\"some\": \"json\"},{\"some\": \"some text here\"},{\"some\": \"some text here\"}\n"
                .as_bytes()
                .to_vec();
        let data1 = Vec::from("{{\"some\": \"json\"},{\"some\": \"some text here\"}\n".as_bytes());
        let new_data = sed_tree_op(&mut rng, &data1, TreeMutate::TreeDup).unwrap();
        println_lossy(&new_data);
        assert_eq!(expected, new_data);
    }

    #[test]
    fn test_tree_swap() {
        // XML Test
        let expected = "<note>  <to>Tove</to> <from>Jani</from> <heading>Reminder</heading> <body>Don't forget me this weekend!</body> </note>\n".as_bytes().to_vec();
        let data2 = Vec::from("<note>  <to>Tove</to> <from>Jani</from> <heading>Reminder</heading> <body>Don't forget me this weekend!</body> </note>\n".as_bytes());
        let mut rng = ChaCha20Rng::seed_from_u64(43);
        let new_data = sed_tree_op(&mut rng, &data2, TreeMutate::TreeSwapPair).unwrap();
        println_lossy(&new_data);
        assert_eq!(expected, new_data);
    }

    #[test]
    fn test_tree_del() {
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        // json
        let expected = "{{\"some\": },{\"some\": \"some text here\"}}\n"
            .as_bytes()
            .to_vec();
        let data1 = Vec::from("{{\"some\": \"json\"},{\"some\": \"some text here\"}}\n".as_bytes());
        let new_data = sed_tree_op(&mut rng, &data1, TreeMutate::TreeDel).unwrap();
        println_lossy(&new_data);
        assert_eq!(expected, new_data);
    }

    #[test]
    fn test_tree_swap_replace() {
        let mut rng = ChaCha20Rng::seed_from_u64(43);
        // json
        let expected = "{{\"some\": {{\"some\": \"json\"},{\"some\": \"some text here\"}}},{\"some\": \"some text here\"}}\n".as_bytes().to_vec();
        let data1 = Vec::from("{{\"some\": \"json\"},{\"some\": \"some text here\"}}\n".as_bytes());
        let new_data = sed_tree_op(&mut rng, &data1, TreeMutate::TreeSwapReplace).unwrap();
        println_lossy(&new_data);
        assert_eq!(expected, new_data);
    }

    #[test]
    fn test_tree_stutter() {
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        // json
        let expected = r#"[{"some": "json"},{"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": {"some": "some text here"}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}]"#.as_bytes().to_vec();
        let data1 = Vec::from("[{\"some\": \"json\"},{\"some\": \"some text here\"}]".as_bytes());
        let new_data = sed_tree_op(&mut rng, &data1, TreeMutate::TreeStutter).unwrap();
        println_lossy(&new_data);
        assert_eq!(expected, new_data);
    }

    #[test]
    fn test_tree_empty() {
        let mut rng = ChaCha20Rng::seed_from_u64(1674713045);
        let data1: Vec<u8> = vec![];
        let new_data = sed_tree_op(&mut rng, &data1, TreeMutate::TreeStutter);
        assert_eq!(None, new_data);
        let new_data = sed_tree_op(&mut rng, &data1, TreeMutate::TreeSwapReplace);
        assert_eq!(None, new_data);
        let new_data = sed_tree_op(&mut rng, &data1, TreeMutate::TreeDel);
        assert_eq!(None, new_data);
        let new_data = sed_tree_op(&mut rng, &data1, TreeMutate::TreeSwapPair);
        assert_eq!(None, new_data);
        let new_data = sed_tree_op(&mut rng, &data1, TreeMutate::TreeDup);
        assert_eq!(None, new_data);
    }
}
