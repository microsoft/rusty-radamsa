//! Checksums used for unique mutations.
//!
use crc::{Crc, CRC_32_CKSUM, CRC_64_REDIS, CRC_82_DARC};
use sha2::{Digest, Sha256, Sha512};
use std::collections::BTreeMap;

// https://reveng.sourceforge.io/crc-catalogue/all.htm
#[derive(Debug, Clone, Copy)]
pub enum HashType {
    Sha,
    Sha256,
    Sha512,
    Crc,
    Crc32, //CRC_32_CKSUM
    Crc64, //CRC_64_REDIS
    Crc82, //CRC_82_DARC
}

pub fn init_digests() -> Vec<Checksum> {
    Vec::from([
        Checksum::new("sha", "Default Hash Sha-256", HashType::Sha),
        Checksum::new("sha256", "Hash Sha-256", HashType::Sha256),
        Checksum::new("sha512", "Hash Sha-512", HashType::Sha512),
        Checksum::new("crc", "Default CRC-64/CKSUM", HashType::Crc64),
        Checksum::new("crc32", "CRC-32/CKSUM", HashType::Crc32),
        Checksum::new("crc64", "CRC-64/REDIS", HashType::Crc64),
        Checksum::new("crc82", "CRC-82/DARC", HashType::Crc82),
    ])
}

pub fn string_digest(_input: &str, _checksums: &mut Vec<Checksum>) -> Option<Checksum> {
    if let Some(c) = _checksums.iter().find(|&x| x.id == _input) {
        return Some(c.clone());
    }
    None
}

pub trait CsDigest {
    fn new_digest() -> Option<Self>
    where
        Self: Sized;
    fn new_crc(_self: &mut Self) -> Option<&mut Self>
    where
        Self: Sized;
    fn updated(&mut self, _data: &[u8]);
    fn finalized(&mut self) -> Option<Box<[u8]>>;
}

impl CsDigest for sha2::Sha256 {
    fn new_digest() -> Option<Self>
    where
        Self: Sized,
    {
        Some(sha2::Sha256::new())
    }
    fn new_crc(_self: &mut Self) -> Option<&mut Self>
    where
        Self: Sized,
    {
        None
    }
    fn updated(&mut self, _data: &[u8]) {
        self.update(_data)
    }
    fn finalized(&mut self) -> Option<Box<[u8]>> {
        let f = sha2::Sha256::finalize(self.clone())
            .to_vec()
            .into_boxed_slice();
        Some(f)
    }
}

impl CsDigest for sha2::Sha512 {
    fn new_digest() -> Option<Self>
    where
        Self: Sized,
    {
        Some(sha2::Sha512::new())
    }
    fn new_crc(_self: &mut Self) -> Option<&mut Self>
    where
        Self: Sized,
    {
        None
    }
    fn updated(&mut self, _data: &[u8]) {
        self.update(_data)
    }
    fn finalized(&mut self) -> Option<Box<[u8]>> {
        let f = sha2::Sha512::finalize(self.clone())
            .to_vec()
            .into_boxed_slice();
        Some(f)
    }
}

pub trait CsDigestB {
    fn updated(&mut self, data: &[u8]);
    fn finalized(&mut self) -> Option<Box<[u8]>>;
}

impl<'a> CsDigestB for crc::Digest<'a, u32> {
    fn updated(&mut self, data: &[u8]) {
        self.update(data);
    }
    fn finalized(&mut self) -> Option<Box<[u8]>> {
        let f = self
            .clone()
            .finalize()
            .to_le_bytes()
            .to_vec()
            .into_boxed_slice();
        Some(f)
    }
}

impl<'a> CsDigestB for crc::Digest<'a, u64> {
    fn updated(&mut self, data: &[u8]) {
        self.update(data);
    }
    fn finalized(&mut self) -> Option<Box<[u8]>> {
        let f = self
            .clone()
            .finalize()
            .to_le_bytes()
            .to_vec()
            .into_boxed_slice();
        Some(f)
    }
}

impl<'a> CsDigestB for crc::Digest<'a, u128> {
    fn updated(&mut self, data: &[u8]) {
        self.update(data);
    }
    fn finalized(&mut self) -> Option<Box<[u8]>> {
        let f = self
            .clone()
            .finalize()
            .to_le_bytes()
            .to_vec()
            .into_boxed_slice();
        Some(f)
    }
}

#[derive(Debug, Clone)]
pub struct Checksum {
    pub id: String,
    pub desc: String,
    pub hash_type: HashType,
}

impl Checksum {
    pub fn new(_id: &str, _desc: &str, _hash_type: HashType) -> Checksum {
        Checksum {
            id: _id.to_string(),
            desc: _desc.to_string(),
            hash_type: _hash_type,
        }
    }
}

#[derive(Debug)]
pub struct Checksums {
    pub checksum: Checksum,
    pub cache: BTreeMap<Box<[u8]>, bool>,
    pub max: usize,
    pub use_hashmap: bool,
}

impl Checksums {
    // new
    pub fn new() -> Checksums {
        Checksums {
            checksum: Checksum::new("sha", "Default Hash Sha-256", HashType::Sha),
            cache: BTreeMap::new(),
            max: 10000, // default,\
            use_hashmap: true,
        }
    }
    pub fn default() -> Checksums {
        Checksums {
            checksum: Checksum::new("sha", "Default Hash Sha-256", HashType::Sha),
            cache: BTreeMap::new(),
            max: 10000, // default,
            use_hashmap: true,
        }
    }
    pub fn add(&mut self, hash: Box<[u8]>) -> Option<bool> {
        if self.cache.contains_key(&hash) {
            // exists
            return Some(true);
        } else {
            if self.cache.len() > self.max {
                return None;
            }
            self.cache.insert(hash, true);
            return Some(false);
        }
    }
    pub fn get_crc<T: CsDigestB>(_digest: &mut T, _data: &Vec<u8>) -> Option<Box<[u8]>> {
        _digest.updated(_data);
        _digest.finalized()
    }
    pub fn get_crc_blocks<T: CsDigestB>(
        _digest: &mut T,
        _data: &Vec<std::boxed::Box<[u8]>>,
    ) -> Option<Box<[u8]>> {
        let mut iter = _data.iter();
        while let Some(block) = iter.next() {
            _digest.updated(&block);
        }
        return _digest.finalized();
    }

    pub fn get_digest<T: CsDigest>(_digest: &mut T, _data: &Vec<u8>) -> Option<Box<[u8]>> {
        _digest.updated(_data);
        _digest.finalized()
    }
    pub fn digest_data(&self, _data: &Vec<u8>) -> Option<Box<[u8]>> {
        match &self.checksum.hash_type {
            HashType::Sha | HashType::Sha256 => {
                let mut d = Sha256::new_digest()?;
                return Self::get_digest(&mut d, _data);
            }
            HashType::Sha512 => {
                let mut d = Sha256::new_digest()?;
                return Self::get_digest(&mut d, _data);
            }
            HashType::Crc32 => {
                let cs = Crc::<u32>::new(&CRC_32_CKSUM);
                let mut d = cs.digest();
                return Self::get_crc(&mut d, _data);
            }
            HashType::Crc | HashType::Crc64 => {
                let cs = Crc::<u64>::new(&CRC_64_REDIS);
                let mut d = cs.digest();
                return Self::get_crc(&mut d, _data);
            }
            HashType::Crc82 => {
                let cs = Crc::<u128>::new(&CRC_82_DARC);
                let mut d = cs.digest();
                return Self::get_crc(&mut d, _data);
            }
        }
    }
    pub fn digest_blocks(&self, _data: Option<&Vec<Box<[u8]>>>) -> Option<Box<[u8]>> {
        if let Some(data) = _data {
            match &self.checksum.hash_type {
                HashType::Sha | HashType::Sha256 | HashType::Sha512 => {
                    let digest: Option<Box<dyn CsDigest>> = match &self.checksum.hash_type {
                        HashType::Sha | HashType::Sha256 => {
                            let h = Sha256::new_digest()?;
                            Some(Box::new(h))
                        }
                        HashType::Sha512 => Some(Box::new(Sha512::new_digest()?)),
                        _ => None,
                    };
                    let mut iter = data.iter();
                    let mut d = digest?;
                    while let Some(block) = iter.next() {
                        d.updated(&block);
                    }
                    return d.finalized();
                }
                HashType::Crc32 => {
                    let cs = Crc::<u32>::new(&CRC_32_CKSUM);
                    let mut d = cs.digest();
                    return Self::get_crc_blocks(&mut d, data);
                }
                HashType::Crc | HashType::Crc64 => {
                    let cs = Crc::<u64>::new(&CRC_64_REDIS);
                    let mut d = cs.digest();
                    return Self::get_crc_blocks(&mut d, data);
                }
                HashType::Crc82 => {
                    let cs = Crc::<u128>::new(&CRC_82_DARC);
                    let mut d = cs.digest();
                    return Self::get_crc_blocks(&mut d, data);
                }
            }
        }
        None
    }
}
