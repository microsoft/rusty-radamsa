//! Mux the generators based on weighted scores.
//!
//! ## GENERATORS:
//!
//! > **DEFAULT:** `random,buffer=10000,file=1000,jump=200,stdin=10000`
//!
//! | id |complete | desc |
//! |----|---------|------|
//! |stdin|&check;|Generator to read data from stdin|
//! |file|&check;|Generator to read data from a file|
//! |tcp|&check;|Generator to read data from a tcp port|
//! |udp|&check;|Generator to read data from a udp port|
//! |buffer|&check;|Generator to read data from buffer|
//! |jump|&cross;|Generator jump streamer|
//! |random|&check;|Generator to make random bytes|
//! |pcapng|&cross;|Generator to generate pcapng data|

use std::fs::File;
use std::io::Cursor;
use std::io::IsTerminal;
use std::io::Read;
use std::io::Write;
use std::io::{self, Seek, SeekFrom};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::path::Path;
#[cfg(test)]
use std::println as debug;

#[cfg(not(test))]
use log::debug;
use log::*;
use print_bytes::println_lossy;
use rand::SeedableRng;
use rand::{Rng, RngCore};
use rand_chacha::ChaCha20Rng;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::shared::*;

// TODO: jump, pcapng
pub const DEFAULT_GENERATORS: &str = "random,buffer=10000,file=1000,jump=200,stdin=10000";
pub const STREAM_SEED_BASE: u128 = 100_000_000_000_000_000_000;
pub const JUMPSTREAM_SEED_BASE: u128 = 0x000f_ffff_ffff;
pub const BUFFER_SEED_BASE: u128 = 42;

#[derive(Debug)]
pub struct Generators {
    pub generators: Vec<Generator>,
    pub generator_nodes: Vec<GenType>,
}

impl Default for Generators {
    fn default() -> Self {
        Self::new()
    }
}

impl Generators {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            generators: Vec::new(),
            generator_nodes: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.generators = init_generators();
    }

    pub fn default_generators(&mut self) {
        self.generator_nodes = string_generators(DEFAULT_GENERATORS, &mut self.generators);
    }

    #[allow(clippy::borrowed_box)]
    pub fn mux_generators(
        &mut self,
        rng: &mut impl Rng,
        paths: &Option<Vec<String>>,
        data: Option<&Box<[u8]>>,
    ) -> Option<&mut Generator> {
        let mut total_priority = 0;
        for generator in &mut self.generators {
            if !self.generator_nodes.contains(&generator.gen_type) {
                generator.priority = 0;
                if let Some(pos) = self
                    .generator_nodes
                    .iter()
                    .position(|r| *r == generator.gen_type)
                {
                    self.generator_nodes.remove(pos);
                }
                continue;
            }
            total_priority += generator.priority;
            generator.init(rng);
            let (paths, data) = match paths {
                Some(p) => {
                    let rng = generator.rng.as_mut().unwrap();
                    let n: usize = p.len().rands(rng);
                    (p.get(n).cloned(), None)
                }
                None => (None, data.cloned()),
            };

            match generator.set_fd(paths, data) {
                Ok(()) => {}
                Err(e) => {
                    error!(
                        "Failed to set fd for {} due to {}.",
                        generator.gen_type.id(),
                        e
                    );
                    generator.priority = 0;
                    if let Some(pos) = self
                        .generator_nodes
                        .iter()
                        .position(|r| *r == generator.gen_type)
                    {
                        self.generator_nodes.remove(pos);
                    }
                }
            }
        }
        self.generators
            .retain(|r| self.generator_nodes.contains(&r.gen_type));
        // TODO: use
        let _initial_priority = total_priority.rands(rng);
        // Sort by priority
        self.generators.sort_by(|x, y| y.priority.cmp(&x.priority));
        //let gen = choose_priority(&mut self.generators, initial_priority)?;
        let generator = self.generators.first_mut()?;
        Some(generator)
    }
}

/// Generator Type
#[derive(Debug, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum GenType {
    Stdin,
    File,
    TCPSocket,
    UDPSocket,
    Buffer,
    Jump,
    Pcapng,
    Random, // stdout
}

#[allow(clippy::enum_glob_use)]
impl GenType {
    #[must_use]
    pub fn id(&self) -> String {
        use GenType::*;
        let name = match *self {
            Stdin => "stdin",
            File => "file",
            TCPSocket => "tcp",
            UDPSocket => "udp",
            Buffer => "buffer",
            Jump => "jump", // not implemented yet
            Pcapng => "pcapng",
            Random => "random", // st
        };
        name.to_string()
    }

    #[must_use]
    pub fn info(&self) -> String {
        use GenType::*;
        let desc = match *self {
            Stdin => "Generator to read data from stdin",
            File => "Generator to read data from a file",
            TCPSocket => "Generator to read data from a tcp port",
            UDPSocket => "Generator to read data from a udp port",
            Buffer => "Generator to read data from buffer",
            Jump => "Generator jump streamer", // not implemented yet
            Pcapng => "Generator to generate pcapng data",
            Random => "Generator to make random bytes",
        };
        desc.to_string()
    }

    pub fn init(
        &self,
        rng: &mut impl Rng,
        path: Option<String>,
        buf: Option<Box<[u8]>>,
    ) -> Result<Box<dyn GenericReader + 'static>, Box<dyn std::error::Error>> {
        match *self {
            Self::Stdin => {
                let stdin = io::stdin();
                if stdin.is_terminal() {
                    return Err(Box::new(NoStdin));
                }
                if buf.is_some() {
                    return Ok(Box::new(Cursor::<Box<[u8]>>::gen_open("r", path, buf)?));
                }
                Ok(Box::new(io::Stdin::gen_open("r", path, buf)?))
            }
            Self::File => Ok(Box::new(File::gen_open("r", path, buf)?)),
            Self::TCPSocket => Ok(Box::new(TcpStream::gen_open("r", path, buf)?)),
            Self::UDPSocket => Ok(Box::new(UdpSocket::gen_open("r", path, buf)?)),
            Self::Buffer => Ok(Box::new(Cursor::<Box<[u8]>>::gen_open("r", path, buf)?)),
            Self::Random => {
                let nblocks = rng.gen_range(1..100);
                let new_rng = ChaCha20Rng::from_rng(rng)?;
                let random_stream = RandomStream {
                    rng: Box::new(new_rng),
                    nblocks,
                };
                Ok(Box::new(random_stream))
            }
            Self::Jump | Self::Pcapng => Err(Box::new(NoneString)),
        }
    }

    #[must_use]
    pub const fn seed(&self) -> u128 {
        use GenType::*;
        match *self {
            Stdin | File | TCPSocket | UDPSocket | Pcapng | Random => STREAM_SEED_BASE,
            Buffer => BUFFER_SEED_BASE,
            Jump => JUMPSTREAM_SEED_BASE, // TODO: not implemented yet
        }
    }
}

#[must_use]
pub fn init_generators() -> Vec<Generator> {
    let mut map = Vec::<Generator>::new();
    for generator in GenType::iter() {
        map.push(Generator::new(generator));
    }
    map
}

pub struct Generator {
    pub priority: usize,
    pub gen_type: GenType,
    pub fd: Option<Box<dyn GenericReader>>,
    pub offset: usize,
    pub block_size: usize,
    pub seed_base: u128,
    pub seed: u64,
    pub rng: Option<Box<dyn RngCore>>,
}

impl PriorityList for Generator {
    fn priority(&self) -> usize {
        self.priority
    }
}

impl std::fmt::Debug for Generator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Generator")
            .field("priority", &self.priority)
            .field("gen_type", &self.gen_type)
            .field("offset", &self.offset)
            .field("block_size", &self.block_size)
            .finish_non_exhaustive()
    }
}

// stream (lets ((rs seed (rand rs 100000000000000000000)))
impl Generator {
    #[must_use]
    pub fn new(gen_type: GenType) -> Self {
        Self {
            priority: 0,
            gen_type,
            fd: None,
            offset: 0,
            block_size: 0,
            seed_base: gen_type.seed(),
            seed: 0,
            rng: None,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn init(&mut self, rng: &mut dyn RngCore) {
        self.seed = self.seed_base.rands(rng) as u64;
        self.rng = Some(Box::new(ChaCha20Rng::seed_from_u64(self.seed)));
        self.block_size = rand_block_size(self.rng.as_mut().unwrap());
        if let Some(fd) = self.fd.as_mut() {
            fd.gen_seek(SeekFrom::Start(0)).ok();
        }
    }

    pub fn set_fd(
        &mut self,
        path: Option<String>,
        buf: Option<Box<[u8]>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = self.rng.as_mut().unwrap().as_mut();
        let fd = self.gen_type.init(&mut rng, path, buf)?;
        self.fd = Some(fd);

        Ok(())
    }

    pub fn next_block(&mut self) -> (Option<Vec<u8>>, bool) {
        let mut buf = vec![0u8; self.block_size];
        if let Some(ref mut fd) = self.fd {
            let n = read_byte_vector(fd, &mut buf, 0).unwrap_or_default();
            if n == 0 {
                return (None, false);
            }
            if n < self.block_size {
                let last_block = Vec::from(&buf[0..n]);
                return (Some(last_block), true);
            }
            self.block_size = rand_block_size(self.rng.as_mut().unwrap());
            return (Some(buf), false);
        }
        (None, false)
    }
}

pub trait GenericReader {
    fn gen_open(
        _permission: &str,
        _path: Option<String>,
        _buf: Option<Box<[u8]>>,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;

    fn gen_read(
        &mut self,
        _buf: &mut Vec<u8>,
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>>;

    fn gen_write(
        &mut self,
        _buf: &[u8],
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>>;

    fn gen_seek(&mut self, _pos: SeekFrom) -> Result<u64, Box<dyn std::error::Error>>;

    fn as_any(&self) -> &dyn std::any::Any;

    fn gen_flush(&mut self) -> Result<usize, Box<dyn std::error::Error>>;
}

impl GenericReader for File {
    fn gen_open(
        permission: &str,
        path: Option<String>,
        _buf: Option<Box<[u8]>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let p = &path.ok_or(NoneString)?;
        match permission {
            "r" => Ok(Self::open(Path::new(p))?),
            "w" => Ok(Self::create(Path::new(p))?),
            _ => Err(Box::new(NoneString)),
        }
    }

    fn gen_read(
        &mut self,
        buf: &mut Vec<u8>,
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.read(buf)?)
    }

    fn gen_write(
        &mut self,
        buf: &[u8],
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.write(buf)?)
    }

    fn gen_seek(&mut self, pos: SeekFrom) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(self.seek(pos)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn gen_flush(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(0)
    }
}

impl GenericReader for io::Stdin {
    fn gen_open(
        _permission: &str,
        _path: Option<String>,
        _buf: Option<Box<[u8]>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(io::stdin())
    }

    fn gen_read(
        &mut self,
        buf: &mut Vec<u8>,
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.lock().read(buf)?)
    }

    fn gen_write(
        &mut self,
        _buf: &[u8],
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        Err(Box::new(NoWrite))
    }

    fn gen_seek(&mut self, _pos: SeekFrom) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(0)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn gen_flush(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(0)
    }
}

impl GenericReader for io::Stdout {
    fn gen_open(
        _permission: &str,
        _path: Option<String>,
        _buf: Option<Box<[u8]>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(io::stdout())
    }

    fn gen_read(
        &mut self,
        _buf: &mut Vec<u8>,
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        Err(Box::new(NoWrite))
    }

    fn gen_write(
        &mut self,
        buf: &[u8],
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        println_lossy(buf);
        Ok(buf.len())
    }

    fn gen_seek(&mut self, _pos: SeekFrom) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(0)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn gen_flush(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(0)
    }
}

impl GenericReader for TcpStream {
    fn gen_open(
        permission: &str,
        path: Option<String>,
        buf: Option<Box<[u8]>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match (path, buf) {
            (Some(path), None) => match permission {
                "r" => {
                    let listener = TcpListener::bind(path)?;
                    debug!("listener {listener:?}");
                    debug!("waiting for stream!");
                    if let Some(Ok(stream)) = listener.incoming().next() {
                        return Ok(stream);
                    }
                    Err(Box::new(NoneString))
                }
                "w" => {
                    let stream = Self::connect(path)?;
                    Ok(stream)
                }
                _ => Err(Box::new(NoneString)),
            },
            _ => Err(Box::new(NoneString)),
        }
    }

    fn gen_read(
        &mut self,
        buf: &mut Vec<u8>,
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        debug!("TCP Gen Read");
        Ok(self.read(buf)?)
    }

    fn gen_write(
        &mut self,
        buf: &[u8],
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        debug!("TCP Gen Write");
        let len = self.write(buf)?;
        self.flush()?;
        Ok(len)
    }

    fn gen_seek(&mut self, _pos: SeekFrom) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(0)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn gen_flush(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(0)
    }
}

impl GenericReader for UdpSocket {
    fn gen_open(
        permission: &str,
        path: Option<String>,
        buf: Option<Box<[u8]>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        match (path, buf) {
            (Some(path), None) => {
                let mut parts = path.split(',');
                let bind_addr = parts.next().unwrap_or("0.0.0.0:8000");
                let connect_addr = parts.next().unwrap_or("127.0.0.1:8000");
                debug!("bind_addr={bind_addr}, connect_addr={connect_addr}");
                match permission {
                    "r" => {
                        let socket = Self::bind(bind_addr)?;
                        let duration = std::time::Duration::new(10, 0);
                        let dur = Some(duration);
                        socket.set_read_timeout(dur)?;
                        Ok(socket)
                    }
                    "w" => {
                        let socket = Self::bind(connect_addr)?;
                        socket.connect(bind_addr)?;
                        Ok(socket)
                    }
                    _ => Err(Box::new(NoneString)),
                }
            }
            _ => Err(Box::new(NoneString)),
        }
    }

    fn gen_read(
        &mut self,
        buf: &mut Vec<u8>,
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let block_len = buf.len();
        let mut cursor = Cursor::new(buf);
        let mut total_len = 0;
        loop {
            let mut buf = vec![0u8; MAX_UDP_PACKET_SIZE];
            let sock = self.try_clone()?;
            match sock.recv_from(&mut buf) {
                Ok((recv_len, src)) => {
                    let mut max_len = block_len;
                    if recv_len < max_len {
                        max_len = recv_len;
                    }
                    let cursor_len = cursor.write(&buf[..max_len])?;
                    total_len += cursor_len;
                    std::thread::spawn(move || {
                        sock.send_to(&buf[..cursor_len], src)
                            .expect("Failed to send a response");
                    });
                    if cursor_len < block_len {
                        break;
                    }
                }
                Err(e) => {
                    error!("couldn't recieve a datagram: {}", e);
                    break;
                }
            }
        }
        Ok(total_len)
    }

    fn gen_write(
        &mut self,
        buf: &[u8],
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let mut cursor = Cursor::new(buf);
        let mut total_len = 0_usize;
        loop {
            let mut cbuff: Vec<u8> = vec![0u8; MAX_UDP_PACKET_SIZE];
            if let Ok(chunk_len) = cursor.read(&mut cbuff) {
                let mut chunk = vec![0u8; chunk_len];
                self.send(&cbuff[..chunk_len])?;
                let (recv_len, _) = self.recv_from(&mut chunk)?;
                if recv_len == 0 {
                    break;
                }
                total_len += recv_len;
                cursor.seek(SeekFrom::Start(total_len as u64))?;
            } else {
                break;
            }
        }
        Ok(buf.len())
    }

    fn gen_seek(&mut self, _pos: SeekFrom) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(0)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn gen_flush(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(0)
    }
}

impl GenericReader for Cursor<Box<[u8]>> {
    fn gen_open(
        _permission: &str,
        _path: Option<String>,
        buf: Option<Box<[u8]>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(buf.map_or_else(|| Err(Box::new(NoWrite)), |b| Ok(Self::new(b)))?)
    }

    fn gen_read(
        &mut self,
        buf: &mut Vec<u8>,
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.read(buf)?)
    }

    fn gen_write(
        &mut self,
        buf: &[u8],
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let len = self.write(buf)?;
        Ok(len)
    }

    fn gen_seek(&mut self, pos: SeekFrom) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(self.seek(pos)?)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    #[allow(clippy::cast_possible_truncation)]
    fn gen_flush(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let pos = self.position() as usize;
        self.seek(SeekFrom::Start(0))?;
        let clear_buf = vec![0u8; pos];
        let len = self.write(&clear_buf)?;
        self.seek(SeekFrom::Start(0))?;
        Ok(len)
    }
}

impl GenericReader for RandomStream {
    fn gen_open(
        _permission: &str,
        _path: Option<String>,
        _buf: Option<Box<[u8]>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Err(Box::new(NoneString))
    }

    fn gen_read(
        &mut self,
        buf: &mut Vec<u8>,
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let size = buf.len();
        if self.nblocks == 0 {
            return Ok(0);
        }
        self.nblocks -= 1;
        let block = random_block(&mut self.rng, size);
        buf.copy_from_slice(&block);
        Ok(buf.len())
    }

    fn gen_write(
        &mut self,
        _buf: &[u8],
        _offset: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(0)
    }

    fn gen_seek(&mut self, _pos: SeekFrom) -> Result<u64, Box<dyn std::error::Error>> {
        Ok(0)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn gen_flush(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(0)
    }
}

// TODO: refactor this for READ trait
pub fn read_byte_vector(
    fd: &mut Box<dyn GenericReader>,
    buf: &mut Vec<u8>,
    offset: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    fd.gen_read(buf, offset)
}

fn rand_block_size(rng: &mut dyn RngCore) -> usize {
    let rand_value = MAX_BLOCK_SIZE.rands(rng);
    if rand_value < MIN_BLOCK_SIZE {
        MIN_BLOCK_SIZE
    } else {
        rand_value
    }
}

pub fn get_fd(
    rng: &mut impl Rng,
    gen_type: &GenType,
    path: Option<String>,
    buf: Option<Box<[u8]>>,
) -> Result<Box<dyn GenericReader + 'static>, Box<dyn std::error::Error>> {
    match *gen_type {
        GenType::Stdin => {
            let stdin = io::stdin();
            if stdin.is_terminal() {
                return Err(Box::new(NoStdin));
            }
            if buf.is_some() {
                return Ok(Box::new(Cursor::<Box<[u8]>>::gen_open("r", path, buf)?));
            }
            Ok(Box::new(io::Stdin::gen_open("r", path, buf)?))
        }
        GenType::File => Ok(Box::new(File::gen_open("r", path, buf)?)),
        GenType::TCPSocket => Ok(Box::new(TcpStream::gen_open("r", path, buf)?)),
        GenType::UDPSocket => Ok(Box::new(UdpSocket::gen_open("r", path, buf)?)),
        GenType::Buffer => Ok(Box::new(Cursor::<Box<[u8]>>::gen_open("r", path, buf)?)),
        GenType::Random => {
            let nblocks = rng.gen_range(1..100);
            let new_rng = ChaCha20Rng::from_rng(rng)?;
            let random_stream = RandomStream {
                rng: Box::new(new_rng),
                nblocks,
            };
            Ok(Box::new(random_stream))
        }
        _ => Err(Box::new(NoneString)),
    }
}

struct RandomStream {
    rng: Box<dyn RngCore>,
    nblocks: usize,
}

#[allow(clippy::cast_sign_loss)]
fn random_block(rng: &mut dyn RngCore, mut n: usize) -> Vec<u8> {
    let mut new_data: Vec<u8> = Vec::new();
    while 0 < n {
        let digit: i128 = rng.r#gen();
        new_data.push((digit & 255) as u8);
        n -= 1;
    }
    new_data
}

/// This function parses generator string i.e. "random,file=1000,jump=200,stdin=100000"
pub fn string_generators(input: &str, generators: &mut [Generator]) -> Vec<GenType> {
    let mut applied_generators: Vec<GenType> = vec![];
    let string_list = input.trim().split(',').collect::<Vec<&str>>();
    for s in string_list {
        let tuple = s.trim().split('=').collect::<Vec<&str>>();
        let gen_id = tuple.first().unwrap_or(&"").trim();
        let priority = tuple
            .get(1)
            .unwrap_or(&"0")
            .trim()
            .parse::<usize>()
            .unwrap_or(0);

        if let Some(generator) = generators.iter_mut().find(|x| x.gen_type.id() == gen_id) {
            generator.priority = if priority < 1 { 1 } else { priority };
            applied_generators.push(generator.gen_type);
        } else {
            panic!("unknown generator {gen_id}");
        }
    }
    applied_generators
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;
    use std::thread;

    use super::*;

    fn filestream() -> PathBuf {
        let base_path = Path::new(".");
        base_path.join("tests").join("filestream.txt")
    }

    fn filestream_str() -> String {
        filestream().into_os_string().into_string().unwrap()
    }

    #[test]
    fn test_read_byte_vector_file() {
        let mut buf = Box::from(vec![0u8; 10]);
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let fd = get_fd(&mut rng, &GenType::File, Some(filestream_str()), None).ok();
        let n = read_byte_vector(&mut fd.unwrap(), &mut buf, 0).ok();
        assert_eq!(n, Some(10));
    }

    #[test]
    fn test_read_byte_vector_file_eof() {
        let mut buf = Box::from(vec![0u8; MAX_BLOCK_SIZE]);
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let fd = get_fd(&mut rng, &GenType::File, Some(filestream_str()), None).ok();
        let n = read_byte_vector(&mut fd.unwrap(), &mut buf, 0).ok();
        assert_eq!(n, Some(3486));
    }

    #[test]
    fn test_read_byte_vector_file_eof_error() {
        let mut buf = Box::from(vec![0u8; MAX_BLOCK_SIZE]);
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let fd = get_fd(&mut rng, &GenType::File, Some(filestream_str()), None).ok();
        let n = read_byte_vector(&mut fd.unwrap(), &mut buf, 44).ok();
        assert_eq!(n, Some(3486));
    }

    #[test]
    fn test_read_byte_vector_large_file_eof() {
        let mut buf = Box::from(vec![0u8; 100]);
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let mut fd = get_fd(&mut rng, &GenType::File, Some(filestream_str()), None).ok();
        let mut n;
        loop {
            n = read_byte_vector(fd.as_mut().unwrap(), &mut buf, 0).unwrap_or_default();
            if n == 0 {
                break;
            }
        }
        assert_eq!(n, 0);
    }

    #[test]
    fn test_read_byte_vector_tcp() {
        let mut buf = Box::from(vec![0u8; 10]);
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let _t = thread::spawn(move || {
            let mut fd: Box<dyn GenericReader> = crate::output::get_fd(
                //&mut rng,
                &crate::output::OutputType::TCPClient,
                Some("127.0.0.1:34254".to_string()),
                &None,
            )
            .unwrap();
            fd.gen_write(&[144], 0).ok();
        });
        let mut tcpstream: Box<dyn GenericReader> = get_fd(
            &mut rng,
            &GenType::TCPSocket,
            Some("127.0.0.1:34254".to_string()),
            None,
        )
        .unwrap();
        let n = read_byte_vector(&mut tcpstream, &mut buf, 0).ok();
        assert_eq!(n, Some(1));
    }

    #[test]
    fn test_read_byte_vector_buff() {
        let mut buf = Box::from(vec![0u8; 10]);
        let data = Box::from(b"Hello World 1 2 3 4 5 6 7 8 9\n" as &[u8]);
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let fd = get_fd(&mut rng, &GenType::Buffer, None, Some(data)).ok();
        let n = read_byte_vector(&mut fd.unwrap(), &mut buf, 0).ok();
        assert_eq!(n, Some(10));
    }

    #[test]
    fn test_read_byte_vector_buff_eof() {
        let mut buf = Box::from(vec![0u8; 40]);
        let data: Box<[u8]> = Box::from(b"Hello World 1 2 3 4 5 6 7 8 9\n" as &[u8]);
        let mut rng = ChaCha20Rng::seed_from_u64(42);
        let fd = get_fd(&mut rng, &GenType::Buffer, None, Some(data)).ok();
        let n = read_byte_vector(&mut fd.unwrap(), &mut buf, 0).ok();
        assert_eq!(n, Some(30));
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_next_block() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha20Rng;
        let file_len = std::fs::metadata(filestream()).unwrap().len() as usize;
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let mut generator = Generator::new(GenType::File);
        generator.init(&mut rng);
        generator.set_fd(Some(filestream_str()), None).ok();
        let mut total_len = 0;
        while let (Some(ref block), _last_block) = generator.next_block() {
            total_len += block.len();
        }
        assert_eq!(total_len, file_len);
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_generators() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha20Rng;
        let file_len = std::fs::metadata(filestream()).unwrap().len() as usize;
        let mut generators = Generators::new();
        generators.init();
        let mut rng = ChaCha20Rng::seed_from_u64(1_675_126_973);
        let paths = vec![filestream_str()];
        generators.generator_nodes = string_generators(
            "random,buffer=10000,file=1000,jump=200",
            &mut generators.generators,
        );
        let mut total_len = 0;
        if let Some(generator) = generators.mux_generators(&mut rng, &Some(paths), None) {
            while let (Some(block), _last_block) = generator.next_block() {
                total_len += block.len();
            }
        }
        assert_eq!(total_len, file_len);
    }

    #[test]
    fn test_random() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha20Rng;
        let mut rng = ChaCha20Rng::seed_from_u64(1_674_713_045);
        let _block = random_block(&mut rng, 300);
    }
}
