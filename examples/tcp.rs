extern crate rusty_radamsa;
use print_bytes::println_lossy;
use rusty_radamsa::Radamsa;
use std::boxed::Box;
use std::thread;

fn main() {
    let _t = thread::spawn(move || {
        let mut fd: Box<dyn rusty_radamsa::generators::GenericReader> =
            rusty_radamsa::output::get_fd(
                &rusty_radamsa::output::OutputType::TCPClient,
                Some("127.0.0.1:34254".to_string()),
                &None,
            )
            .unwrap();
        let _len = fd.gen_write(&[41u8; 20], 0);
    });
    let mut r = Radamsa::new_with_seed(1);
    r.init();
    r.set_mutators("default").expect("bad input");
    r.set_generators("tcp").expect("bad input");
    r.set_patterns("default").expect("bad input");
    r.set_output(vec!["buffer"]).expect("bad input");
    let mut out_buffer: Box<[u8]> = Box::from(vec![0u8; 2048]);
    let paths = vec!["127.0.0.1:34254".to_string()];
    let _len = r.fuzz(None, Some(paths), Some(&mut out_buffer)).unwrap();
    println_lossy(&out_buffer[.._len]);
}
