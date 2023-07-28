#![doc = include_str!("../README.md")]
#[macro_use]
extern crate lazy_static;

pub mod digest;
mod fuse;
pub mod generators;
mod generic;
pub mod mutations;
pub mod output;
pub mod patterns;
pub mod shared;
mod split;
pub mod utils;

use crate::shared::time_seed;
use crate::shared::BadInput;
use log::*;
use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::boxed::Box;
use std::ffi::CStr;

#[cfg(not(test))]
use log::debug;

#[cfg(test)]
use std::println as debug;

/// Initial radamsa configs
pub struct Radamsa {
    /// Random seed (number, default random).
    pub seed: u64,
    /// user specified random generator, default ChaCha20Rng.
    pub rng: Box<dyn RngCore>,
    /// show progress during generation. Set RUST_LOG env variable.
    pub verbose: bool,
    /// how many outputs to generate (number or inf).
    pub count: usize,
    /// start from given testcase (TODO: implement).
    pub offset: usize,
    /// sleep for n milliseconds between outputs (TODO: implement).
    pub delay: usize,
    /// maximum number of checksums in uniqueness filter (0 disables).
    /// hash algorithm for uniqueness checks (stream, sha1 or sha256).
    pub(crate) checksums: digest::Checksums,
    /// Which mutations to use. default default_mutations.
    pub(crate) mutations: mutations::Mutations,
    /// which patterns to use. default default_patterns.
    pub(crate) patterns: patterns::Patterns,
    /// which data generators to use. default "random,file=1000,jump=200,stdin=100000".
    pub(crate) generators: generators::Generators,
    /// Contains the outputs in which the fuzzer writes to.
    pub(crate) outputs: output::Outputs,
}

impl std::fmt::Debug for Radamsa {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Radamsa")
            .field("seed", &self.seed)
            .field("verbose", &self.verbose)
            .field("count", &self.count)
            .field("offset", &self.offset)
            .field("delay", &self.delay)
            .field("checksums", &self.checksums)
            .field("mutations", &self.mutations)
            .field("patterns", &self.patterns)
            .field("generators", &self.generators)
            .field("outputs", &self.outputs)
            .finish()
    }
}

impl Radamsa {
    /// Constructs a new radamsa object with uninitialized contents.
    /// The seed for rand is based on time. See [time_seed].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new();
    /// ```
    pub fn new() -> Radamsa {
        let seed = time_seed();
        Radamsa {
            seed: seed,
            rng: Box::new(ChaCha20Rng::seed_from_u64(seed)),
            verbose: false,
            count: 0,
            offset: 0,
            delay: 0,
            checksums: digest::Checksums::new(),
            mutations: mutations::Mutations::new(),
            patterns: patterns::Patterns::new(),
            generators: generators::Generators::new(),
            outputs: output::Outputs::new(),
        }
    }

    /// Constructs a new radamsa object with uninitialized contents except for
    /// the user provided seed.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new_with_seed(42);
    /// ```
    pub fn new_with_seed(_seed: u64) -> Radamsa {
        Radamsa {
            seed: _seed,
            rng: Box::new(ChaCha20Rng::seed_from_u64(_seed)),
            verbose: false,
            count: 0,
            offset: 0,
            delay: 0,
            checksums: digest::Checksums::new(),
            mutations: mutations::Mutations::new(),
            patterns: patterns::Patterns::new(),
            generators: generators::Generators::new(),
            outputs: output::Outputs::new(),
        }
    }
    /// Initializes available generators, mutations, patterns, and outputs.
    pub fn init(&mut self) {
        self.generators.init();
        self.mutations.init();
        self.patterns.init();
        self.outputs.init();
    }

    /// Initializes with defaults for patterns, mutations, patterns, and outputs.
    /// A time generated u64 value is used for the seed.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::default();
    /// ```
    pub fn default() -> Radamsa {
        let mut r = Self::new();
        r.init();
        r.generators.default_generators();
        r.mutations.default_mutations();
        r.patterns.default_patterns();
        r.mutations.randomize(&mut r.rng);
        r.outputs.default_outputs();
        r
    }

    /// Initializes with defaults for patterns, mutations, patterns, and outputs.
    /// Seed is used to initialize rand.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::default_with_seed(42);
    /// ```
    pub fn default_with_seed(_seed: u64) -> Radamsa {
        let mut r = Self::new_with_seed(_seed);
        r.verbose = true;
        r.init();
        r.generators.default_generators();
        r.mutations.default_mutations();
        r.patterns.default_patterns();
        r.mutations.randomize(&mut r.rng);
        r.outputs.default_outputs();
        r
    }

    /// Take only first n bytes of each output (mainly intended for UDP).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new();
    /// rad.init();
    /// rad.truncate(100);
    /// rad.set_generators("file");
    /// rad.set_mutators("default");
    /// rad.set_patterns("default");
    /// rad.set_output(vec!["buffer"]);
    /// let paths = vec![".\\tests\\filestream.txt".to_string()];
    /// let mut out_buffer = std::boxed::Box::from(vec![0u8; 2048]);
    /// let len = rad.fuzz(None, Some(paths), Some(&mut out_buffer)).unwrap_or(0);
    /// assert_eq!(len, 100)
    /// ```
    pub fn fuzz(
        &mut self,
        _data: Option<&Box<[u8]>>,
        _paths: Option<Vec<String>>,
        _buffer: Option<&mut Box<[u8]>>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut _out_len = self.outputs.truncate;
        debug!(
            "Available generators {:?}",
            self.generators
                .generators
                .iter()
                .map(|o| o.gen_type.id())
                .collect::<Vec<String>>()
        );
        let mut n = 1;
        let mut p = 0;
        let mut buffer = match _buffer {
            Some(b) => Some(b),
            None => None,
        };
        self.outputs.init_pipes(&buffer)?;
        // Initial pass
        let generator = self
            .generators
            .mux_generators(&mut self.rng, &_paths, _data)
            .expect("Failed to choose generator, paths maybe malformed");
        let (og_data, mut mut_data) = self
            .patterns
            .mux_patterns(generator, &mut self.mutations)
            .unwrap();

        if !self.checksums.use_hashmap {
            _out_len = self.outputs.mux_output(&mut_data, &mut buffer)?;
        } else {
            loop {
                let cs_exists = match self.checksums.digest_data(&mut_data) {
                    Some(cs) => self.checksums.add(cs).unwrap_or(true),
                    None => false,
                };
                if cs_exists {
                    if p >= crate::shared::MAX_CHECKSUM_RETRY {
                        error!("max unique reached");
                        // Make sure to return something
                        _out_len = self.outputs.mux_output(&mut_data, &mut buffer)?;
                        break;
                    }
                    // Try again
                    let generator = self
                        .generators
                        .mux_generators(&mut self.rng, &_paths, Some(&og_data))
                        .unwrap();
                    match self.patterns.mux_patterns(generator, &mut self.mutations) {
                        Some((_, m)) => mut_data = m,
                        None => {}
                    }
                    p += 1;
                    debug!("in count loop");
                    continue;
                } else {
                    // Successful unique value
                    _out_len = self.outputs.mux_output(&mut_data, &mut buffer)?;
                    p = 0;
                    if n < 1 {
                        break;
                    } else if n < self.count {
                        n += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(_out_len)
    }

    /// Sets the generators to be used.
    /// For list of generators see [generators].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new();
    /// rad.init();
    /// rad.set_generators("file");
    /// ```
    pub fn set_generators(&mut self, _gen: &str) -> Result<(), Box<dyn std::error::Error>> {
        if _gen == "default" {
            self.generators.default_generators();
        } else {
            self.generators.generator_nodes =
                crate::generators::string_generators(_gen, &mut self.generators.generators);
        }

        if self.generators.generator_nodes.is_empty() {
            Err(Box::new(BadInput))
        } else {
            Ok(())
        }
    }

    /// Sets the mutators to be used.
    /// For list of mutators see [mutations].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new();
    /// rad.init();
    /// rad.set_mutators("bd=3,bf,num=2");
    /// ```
    pub fn set_mutators(&mut self, _mut: &str) -> Result<(), Box<dyn std::error::Error>> {
        if _mut == "default" {
            self.mutations.default_mutations();
        } else {
            self.mutations.mutator_nodes =
                crate::mutations::string_mutators(_mut, &mut self.mutations.mutators);
        }
        if self.mutations.mutator_nodes.is_empty() {
            Err(Box::new(BadInput))
        } else {
            Ok(())
        }
    }

    /// Sets the patterns to be used.
    /// For list of patterns see [patterns].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new();
    /// rad.init();
    /// rad.set_patterns("od");
    /// ```
    pub fn set_patterns(&mut self, _pat: &str) -> Result<(), Box<dyn std::error::Error>> {
        if _pat == "default" {
            self.patterns.default_patterns();
        } else {
            self.patterns.pattern_nodes =
                crate::patterns::string_patterns(_pat, &mut self.patterns.patterns);
        }
        if self.patterns.pattern_nodes.is_empty() {
            Err(Box::new(BadInput))
        } else {
            Ok(())
        }
    }

    /// Sets the outputs to be used.
    /// For list of outputs see [output].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new();
    /// rad.init();
    /// rad.set_output(vec!["-"]);
    /// rad.set_output(vec!["file","tmp.bin"]);
    /// ```
    pub fn set_output(&mut self, _out: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
        if _out == vec!["default"] {
            self.outputs.default_outputs();
        } else {
            self.outputs.outputs = crate::output::string_outputs(_out, &mut self.outputs.outputs);
        }
        if self.outputs.outputs.is_empty() {
            Err(Box::new(BadInput))
        } else {
            Ok(())
        }
    }

    /// Sets the checksum type to be used.
    /// For list of checksum types see [digest].
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new();
    /// rad.init();
    /// rad.set_checksum("sha");
    /// ```
    pub fn set_checksum(&mut self, _chk: &str) -> Result<(), Box<dyn std::error::Error>> {
        if _chk != "default" {
            if let Some(digest) = crate::digest::string_digest(_chk, &mut digest::init_digests()) {
                self.checksums.checksum = digest;
                return Ok(());
            } else {
                return Err(Box::new(BadInput));
            }
        }
        Ok(())
    }

    /// Sets the maximum unique checksums in hashmap.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new();
    /// rad.init();
    /// rad.checksum_max(10000);
    /// ```
    pub fn checksum_max(&mut self, _max: usize) {
        self.checksums.max = _max;
    }

    /// Take only first n bytes of each output (mainly intended for UDP).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut rad = rusty_radamsa::Radamsa::new();
    /// rad.init();
    /// rad.truncate(10);
    /// rad.set_generators("random");
    /// rad.set_mutators("default");
    /// rad.set_patterns("default");
    /// rad.set_output(vec!["buffer"]);
    /// let mut out_buffer = std::boxed::Box::from(vec![0u8; 2048]);
    /// let len = rad.fuzz(None, None, Some(&mut out_buffer)).unwrap_or(0);
    /// assert_eq!(len, 10)
    /// ```
    pub fn truncate(&mut self, _size: usize) {
        self.outputs.truncate = _size;
    }
    pub fn set_seed(&mut self, _seed: u64) {
        self.seed = _seed;
        self.rng = Box::new(ChaCha20Rng::seed_from_u64(_seed));
    }
    pub fn resize(&mut self, _enable: bool) {
        self.outputs.resize = _enable;
    } 
    pub fn enable_hashmap(&mut self, _enable: bool) {
        self.checksums.use_hashmap = _enable;
    }
}

/// Read data from a Box and write output to target Box and return the amount
/// of data written.
///
/// # Arguments
///
/// * `_data` - &Box<[u8]> input data.
/// * `_len` - Length of input data.
/// * `_target` - &mut Box<[u8]> output data.
/// * `_max` - Maximum size of data. Can be used to truncate data.
/// * `_seed` - u64 seed for the rand generator.
///
/// # Examples
///
/// ```
/// fn main() {
///     let data = std::boxed::Box::from("1 2 3 4 5 6 7 8 9 10 11 12\n".as_bytes());
///     let mut out_buffer = std::boxed::Box::from(vec![0u8; 2048]);
///     let max_len = 100;
///     let seed: u64 = 42;
///     let _len = rusty_radamsa::radamsa(&data, data.len(), &mut out_buffer, max_len, seed);
///     println!("{:?}", out_buffer);
/// }
/// ```
pub fn radamsa(
    _data: &Box<[u8]>,
    _len: usize,
    _target: &mut Box<[u8]>,
    _max: usize,
    _seed: u64,
) -> usize {
    let mut r = match _seed {
        0 => Radamsa::default(),
        _ => Radamsa::default_with_seed(_seed),
    };
    r.set_generators("buffer").ok();
    r.set_output(vec!["buffer"]).ok();
    r.truncate(_max);
    debug!("Seed {}", r.seed);
    r.fuzz(Some(_data), None, Some(_target)).unwrap_or(0)
}

/// This C FFI function uses the seed 42 to initialize.
///  
/// Generators used are buffer and random.
/// Default output is set to buffer.
/// The use of hashmapping output is disabled.
///
/// # Examples
///
/// ```text
/// #include "rusty_radamsa.h"
/// void *radamsa_handle = NULL;
///
/// extern "C" int LLVMFuzzerInitialize(int *argc, char ***argv) {
///     radamsa_handle = rusty_radamsa_init();
///     return 0;
/// }
/// ```
#[no_mangle]
pub extern "C" fn rusty_radamsa_init() -> *mut Radamsa {
    let mut r = Radamsa::new_with_seed(42);
    r.checksums.use_hashmap = false;
    r.init();
    r.set_generators("buffer=1000,random").ok();
    r.set_mutators("default").ok();
    r.set_patterns("default").ok();
    r.mutations.randomize(&mut r.rng);
    r.set_output(vec!["buffer"]).ok();
    Box::into_raw(Box::new(r))
}

/// This C FFI function is used to set the mutator string to customize mutators.
///
/// # Examples
///
/// ```text
/// #include "rusty_radamsa.h"
/// void *radamsa_handle = NULL;
///
/// extern "C" int LLVMFuzzerInitialize(int *argc, char ***argv) {
///     radamsa_handle = rusty_radamsa_init();
///     rusty_radamsa_set_mutator(radamsa_handle, (const uint8_t*)"default");
///     return 0;
/// }
/// ```
#[no_mangle]
pub extern "C" fn rusty_radamsa_set_mutator(ctx: *mut Radamsa, config: *const i8) {
    unsafe {
        let radamsa_instance = &mut *ctx;
        let c_str: &CStr = CStr::from_ptr(config);
        let mutator_str: &str = c_str.to_str().unwrap();
        let owned_mutator_str = mutator_str.to_owned();
        radamsa_instance.set_mutators(&owned_mutator_str).ok();
    }
}

/// This C FFI function does the actual fuzzing.
///
/// A seed is required to produce new a new rand number.
/// The maximum size is truncating the output.
///
/// # Examples
///
/// ```text
/// #include "rusty_radamsa.h"
/// void *radamsa_handle = NULL;
///
/// extern "C" int LLVMFuzzerInitialize(int *argc, char ***argv) {
///     radamsa_handle = rusty_radamsa_init();
///     rusty_radamsa_set_mutator(radamsa_handle, (const uint8_t*)"default");
///     return 0;
/// }
/// extern "C" size_t LLVMFuzzerCustomMutator(uint8_t *Data, size_t Size,
///      size_t MaxSize, unsigned int Seed) {
///    size_t NewSize = rusty_radamsa(radamsa_handle, Data, Size, Data, MaxSize, Seed);
///    return NewSize;
/// }
/// ```
#[no_mangle]
pub extern "C" fn rusty_radamsa(
    ctx: *mut Radamsa,
    data: *const u8,
    size: usize,
    out: *mut u8,
    max_size: usize,
    seed: u64,
) -> usize {
    unsafe {
        let radamsa_instance = &mut *ctx;
        let bytes = std::slice::from_raw_parts(data, size);
        let in_data = Box::<[u8]>::from(bytes);
        let out = std::slice::from_raw_parts_mut(out, max_size);
        let mut out_buffer = Box::<[u8]>::from(vec![0u8; max_size]);
        radamsa_instance.truncate(max_size);
        radamsa_instance.set_seed(seed);
        let result = radamsa_instance
            .fuzz(Some(&in_data), None, Some(&mut out_buffer))
            .unwrap_or(0);
        out[..out_buffer.len()].copy_from_slice(&out_buffer);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use print_bytes::println_lossy;

    #[test]
    fn test_digests() {
        let mut r = Radamsa::new_with_seed(1684207108);
        debug!("seed {}", r.seed);
        r.count = 30;
        r.init();
        r.set_mutators("bd=3,bf,nop,num=2").expect("bad input");
        r.set_generators("default").expect("bad input");
        r.set_patterns("default").expect("bad input");
        let data: Box<[u8]> = Box::from("ABCDEFG 12345".as_bytes());
        let mut output = vec![0u8; 20].into_boxed_slice();
        r.fuzz(Some(&data), None, Some(&mut output)).unwrap();
        let expected = vec![
            65, 66, 67, 68, 69, 70, 71, 32, 45, 53, 52, 51, 51, 50, 53, 50, 48, 49, 55, 54,
        ];
        println_lossy(&output.to_vec());
        assert_eq!(output.to_vec(), expected);
    }

    #[test]
    fn test_radamsa() {
        let data = Box::from("ABC 1 2 3 4 5 6 7 8 9 10 11 12\n".as_bytes());
        let _expected: Vec<u8> = vec![
            65, 66, 67, 32, 49, 32, 50, 32, 51, 32, 52, 32, 53, 32, 54, 32, 55, 32, 56, 32, 57, 32,
            49, 48, 32, 49, 49, 32, 49, 50, 57, 10,
        ];
        let mut out_buffer = Box::from(vec![0u8; 2048]);
        let max_len = 2048;
        let seed: u64 = 42;
        println!("Seed {}", seed);
        let _len = radamsa(&data, data.len(), &mut out_buffer, max_len, seed);
        println_lossy(&out_buffer.to_vec());
        assert_eq!(&out_buffer[.._len], &*_expected);
    }

    #[test]
    fn test_lib_tcp() {
        use std::boxed::Box;
        use std::thread;
        let _t = thread::spawn(move || {
            let mut fd: Box<dyn crate::generators::GenericReader> = crate::output::get_fd(
                &crate::output::OutputType::TCPClient,
                Some("127.0.0.1:8000".to_string()),
                &None,
            )
            .unwrap();
            let len = fd.gen_write(&[3u8; 20], 0);
            debug!("wrote {:?}", len);
        });
        let mut r = Radamsa::new_with_seed(1);
        r.init();
        r.set_mutators(
            "ft=2,fn,num=5,ld,lds,lr2,li,ls,lp,lr,sr,sd,bd,bf,bi,br,bp,bei,bed,ber,uw,ui=2",
        )
        .expect("bad input");
        r.set_generators("tcp").expect("bad input");
        r.set_patterns("default").expect("bad input");
        r.set_output(vec!["buffer"]).expect("bad input");
        let mut out_buffer: Box<[u8]> = Box::from(vec![0u8; 2048]);
        let paths = crate::shared::_vec_of_strings!["127.0.0.1:8000"];
        let _len = r.fuzz(None, Some(paths), Some(&mut out_buffer)).unwrap();
        debug!("test len {}", _len);
        println_lossy(&out_buffer.to_vec());
        let _expected: Vec<u8> = vec![3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 78, 3, 3, 3, 3, 3, 3, 3, 3];
        assert_eq!(&out_buffer[.._len], &*_expected);
    }

    #[test]
    fn test_truncate() {
        let mut rad = Radamsa::default();
        rad.init();
        rad.truncate(10);
        rad.set_generators("random").ok();
        rad.set_mutators("default").ok();
        rad.set_patterns("default").ok();
        rad.set_output(vec!["buffer"]).ok();
        let mut out_buffer = std::boxed::Box::from(vec![0u8; 2048]);
        let len = rad.fuzz(None, None, Some(&mut out_buffer)).unwrap_or(0);
        assert_eq!(len, 10)
    }
}
