//! Mux the patterns based on weighted scores.
//!

use std::boxed::Box;
#[cfg(test)]
use std::println as debug;

#[cfg(not(test))]
use log::debug;
use rand::RngCore;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::generators::Generator;
use crate::mutations::Mutations;
use crate::shared::*;

pub const DEFAULT_PATTERNS: &str = "od,nd=2,bu";

pub type PatternFunc =
    fn(_gen: &mut Generator, _mutas: &mut Mutations) -> Option<(Box<[u8]>, Vec<u8>)>;

#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    OnceDec,
    ManyDec,
    Burst,
}

#[allow(clippy::enum_glob_use)]
impl PatternType {
    #[must_use]
    pub fn id(&self) -> String {
        use PatternType::*;
        let id = match *self {
            OnceDec => "od",
            ManyDec => "nd",
            Burst => "bu",
        };
        id.to_string()
    }

    #[must_use]
    pub fn info(&self) -> String {
        use PatternType::*;
        let info = match *self {
            OnceDec => "Mutate once",
            ManyDec => "Mutate possibly many times",
            Burst => "Make several mutations closeby once",
        };
        info.to_string()
    }

    fn apply(
        self,
        generator: &mut Generator,
        mutas: &mut Mutations,
    ) -> Option<(Box<[u8]>, Vec<u8>)> {
        use PatternType::*;
        match self {
            OnceDec => pat_once_dec(generator, mutas),
            ManyDec => pat_many_dec(generator, mutas),
            Burst => pat_burst(generator, mutas),
        }
    }
}

#[derive(Debug)]
pub struct Patterns {
    /// These are the Pattern structures that are implemented.
    pub patterns: Vec<Pattern>,
    /// These are the string pattern ids chosen by the user.
    pub pattern_nodes: Vec<PatternType>,
}

#[allow(clippy::new_without_default)]
impl Patterns {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            patterns: Vec::new(),
            pattern_nodes: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.patterns = init_patterns();
    }

    pub fn default_patterns(&mut self) {
        self.pattern_nodes = string_patterns(DEFAULT_PATTERNS, &mut self.patterns);
    }

    /// Will choose the top priority Pattern. Pattern will execute and return mutator content.
    pub fn mux_patterns(
        &mut self,
        generator: &mut Generator,
        mutas: &mut Mutations,
    ) -> Option<(Box<[u8]>, Vec<u8>)> {
        // (og, mutated)
        let rng = generator.rng.as_mut().unwrap();
        // weighted-permutation
        let mut total_priority = 0;
        for pattern in &self.patterns {
            total_priority += pattern.priority;
        }
        let initial_priority = total_priority.rands(rng);
        // Sort by priority
        self.patterns.sort_by(|y, x| x.priority.cmp(&y.priority));
        // choose-pri
        let chosen_pattern = choose_priority(&mut self.patterns, initial_priority)?;
        debug!("pat {}", chosen_pattern.pattern_type.id());
        chosen_pattern.pattern_type.apply(generator, mutas)
    }
}

impl PriorityList for Pattern {
    fn priority(&self) -> usize {
        self.priority
    }
}

#[derive(Debug)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub priority: usize,
}

impl Pattern {
    const fn new(pattern: PatternType) -> Self {
        Self {
            pattern_type: pattern,
            priority: 0,
        }
    }
}

#[must_use]
pub fn init_patterns() -> Vec<Pattern> {
    let mut list = Vec::<Pattern>::new();
    for pattern in PatternType::iter() {
        list.push(Pattern::new(pattern));
    }
    list
}

/// This function parses mutation string i.e. ft=2,fo=2
pub fn string_patterns(input: &str, patterns: &mut [Pattern]) -> Vec<PatternType> {
    let mut applied_patterns: Vec<PatternType> = vec![];
    let string_list = input.trim().split(',').collect::<Vec<&str>>();
    for s in string_list {
        let tuple = s.trim().split('=').collect::<Vec<&str>>();
        let pattern_id = tuple.first().unwrap_or(&"").trim();
        let priority = tuple
            .get(1)
            .unwrap_or(&"0")
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        if let Some(pattern) = patterns
            .iter_mut()
            .find(|x| x.pattern_type.id() == pattern_id)
        {
            pattern.priority = if priority < 1 { 1 } else { priority };
            applied_patterns.push(pattern.pattern_type);
        } else {
            panic!("unknown mutator {pattern_id}");
        }
    }
    applied_patterns
}

pub fn pat_once_dec(
    generator: &mut Generator,
    mutas: &mut Mutations,
) -> Option<(Box<[u8]>, Vec<u8>)> {
    // Mutate once
    let (og_data, mut data) = mutate_once(generator, mutas);
    let mut new_data: Vec<u8> = vec![];
    for x in &mut data {
        new_data.append(x);
    }
    Some((og_data, new_data))
}

/// 1 or more mutations
pub fn pat_many_dec(
    generator: &mut Generator,
    mutas: &mut Mutations,
) -> Option<(Box<[u8]>, Vec<u8>)> {
    // Mutate once
    let (og_data, mut data) = mutate_once(generator, mutas);
    let mut _count = 0_usize;
    let mut mut_data: Option<Vec<Vec<u8>>> = None;
    while rand_occurs(generator.rng.as_mut()?, REMUTATE_PROBABILITY) {
        mut_data = Some(mutate_multi(generator.rng.as_mut()?, &data, mutas));
        _count += 1;
    }
    let mut new_data: Vec<u8> = vec![];
    match mut_data {
        Some(mut d) => {
            for x in &mut d {
                new_data.append(x);
            }
        }
        None => {
            for x in &mut data {
                new_data.append(x);
            }
        }
    }
    Some((og_data, new_data))
}

pub fn pat_burst(generator: &mut Generator, mutas: &mut Mutations) -> Option<(Box<[u8]>, Vec<u8>)> {
    let (og_data, mut data) = mutate_once(generator, mutas);
    let mut _count = 0_usize;
    let mut n = 1;
    loop {
        let p = rand_occurs(generator.rng.as_mut().unwrap(), REMUTATE_PROBABILITY);
        if p || n < 2 {
            data = mutate_multi(generator.rng.as_mut().unwrap(), &data, mutas);
            n += 1;
            _count += 1;
        } else {
            break;
        }
    }
    let mut new_data: Vec<u8> = vec![];
    for x in &mut data {
        new_data.append(x);
    }
    Some((og_data, new_data))
}

fn mutate_multi(rng: &mut dyn RngCore, data: &Vec<Vec<u8>>, mutas: &mut Mutations) -> Vec<Vec<u8>> {
    let mut ip = INITIAL_IP.rands(rng);
    let mut output: Vec<Vec<u8>> = Vec::new();
    for data in data {
        let n = ip.rands(rng);
        if n == 0 {
            if let Some(new_data) = mutas.mux_fuzzers(rng, Some(data)) {
                output.push(new_data);
                ip += 1;
            } else {
                output.push(data.clone());
            }
        } else {
            output.push(data.clone());
        }
    }
    output
}

#[allow(clippy::type_complexity)]
fn mutate_once(
    //_rng: &mut dyn RngCore,
    generator: &mut Generator,
    mutas: &mut Mutations,
) -> (Box<[u8]>, Vec<Vec<u8>>) {
    // initial inverse probability
    let mut ip = INITIAL_IP.rands(generator.rng.as_mut().unwrap());
    let mut og_output: Vec<u8> = Vec::new();
    let mut new_output: Vec<Vec<u8>> = Vec::new();
    while let (Some(ref data), last_block) = generator.next_block() {
        og_output.append(&mut data.clone());
        let n = ip.rands(generator.rng.as_mut().unwrap());
        if n == 0 || last_block {
            if let Some(new_data) = mutas.mux_fuzzers(generator.rng.as_mut().unwrap(), Some(data)) {
                new_output.push(new_data);
                ip += 1;
            } else {
                new_output.push(data.clone());
            }
        } else {
            new_output.push(data.clone());
        }
    }
    (og_output.into_boxed_slice(), new_output)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::path::PathBuf;

    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    use super::*;
    use crate::vec_of_strings;

    fn filestream() -> PathBuf {
        let base_path = Path::new(".");
        base_path.join("tests").join("filestream.txt")
    }

    fn filestream_str() -> String {
        filestream().into_os_string().into_string().unwrap()
    }

    /// Test stream data
    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_mutate_once() {
        //let path = ".\\tests\\filestream.txt".to_string();
        let file_len = std::fs::metadata(filestream()).unwrap().len() as usize;
        let mut generators = crate::generators::Generators::new();
        generators.init();
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        let paths = vec_of_strings![filestream_str()];
        generators.generator_nodes =
            crate::generators::string_generators("file=200", &mut generators.generators);
        let mut patterns = Patterns::new();
        let mut mutations = Mutations::new();
        mutations.init();
        patterns.init();
        mutations.mutator_nodes =
            crate::mutations::string_mutators("num,br", &mut mutations.mutators);
        patterns.pattern_nodes = string_patterns("od", &mut patterns.patterns);
        mutations.randomize(&mut rng);
        let total_len = generators
            .mux_generators(&mut rng, &Some(paths), None)
            .map_or(0, |generator| {
                let (_og_data, new_data) =
                    patterns.mux_patterns(generator, &mut mutations).unwrap();
                new_data.len()
            });
        debug!("file_len {file_len}");
        assert_eq!(total_len, 3487);
    }

    #[test]
    fn test_mutate_multiple() {
        let mut generators = crate::generators::Generators::new();
        generators.init();
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        let paths = vec_of_strings![filestream_str()];
        generators.generator_nodes =
            crate::generators::string_generators("file=200", &mut generators.generators);
        let mut patterns = Patterns::new();
        let mut mutations = Mutations::new();
        mutations.init();
        patterns.init();
        mutations.mutator_nodes =
            crate::mutations::string_mutators("num,bd", &mut mutations.mutators);
        patterns.pattern_nodes = string_patterns("nd", &mut patterns.patterns);
        mutations.randomize(&mut rng);
        let total_len = generators
            .mux_generators(&mut rng, &Some(paths), None)
            .map_or(0, |generator| {
                let (_og_data, new_data) =
                    patterns.mux_patterns(generator, &mut mutations).unwrap();
                new_data.len()
            });
        assert_eq!(total_len, 3485);
    }

    #[test]
    fn test_mutate_burst() {
        let mut generators = crate::generators::Generators::new();
        generators.init();
        let mut rng = ChaCha20Rng::seed_from_u64(1);
        let paths = vec_of_strings![filestream_str()];
        generators.generator_nodes =
            crate::generators::string_generators("file=200", &mut generators.generators);
        let mut patterns = Patterns::new();
        let mut mutations = Mutations::new();
        mutations.init();
        patterns.init();
        mutations.mutator_nodes =
            crate::mutations::string_mutators("num=3,br=2,bd=1", &mut mutations.mutators);
        patterns.pattern_nodes = string_patterns("bu", &mut patterns.patterns);
        mutations.randomize(&mut rng);
        let total_len = generators
            .mux_generators(&mut rng, &Some(paths), None)
            .map_or(0, |generator| {
                let (_og_data, new_data) =
                    patterns.mux_patterns(generator, &mut mutations).unwrap();
                new_data.len()
            });
        assert_eq!(total_len, 3524);
    }
}
