extern crate rusty_radamsa;
use print_bytes::println_lossy;

fn main() {
    let data = Box::from("1 2 3 4 5 6 7 8 9 10 11 12\n".as_bytes());
    let _expected: Vec<u8> = vec![49, 32, 50, 32, 51, 32, 52, 32, 53, 32];
    let mut out_buffer = Box::from(vec![0u8; 2048]);
    let max_len = 100;
    let seed: u64 = 42;
    let _len = rusty_radamsa::radamsa(&data, data.len(), &mut out_buffer, max_len, seed);
    println_lossy(&*out_buffer);
}
