extern crate rusty_radamsa;
use print_bytes::println_lossy;
use rusty_radamsa::Radamsa;
use std::boxed::Box;

fn main() {
    let data = Box::from("1 2 3 4 5 6 7 8 9 10 11 12\n".as_bytes());
    let _expected: Vec<u8> = vec![49, 32, 50, 32, 51, 32, 52, 32, 53, 32];
    let mut out_buffer = Box::from(vec![0u8; 0]);
    let max_len = 100;
    let seed: u64 = 42;
    let mut r = Radamsa::new_with_seed(1);
    r.init();
    r.set_mutators("default").expect("bad input");
    r.set_generators("buffer").expect("bad input");
    r.set_patterns("default").expect("bad input");
    r.set_output(vec!["buffer"]).expect("bad input");
    r.resize(true);
    let _len = r.fuzz(Some(&data), None, Some(&mut out_buffer)).unwrap();
    println_lossy(&out_buffer[..out_buffer.len()]);
}
