//! Checksums used for unique mutations.
//!
use std::collections::BTreeMap;

use crc::{Crc, CRC_32_CKSUM, CRC_64_REDIS, CRC_82_DARC};
use sha2::{Digest, Sha256, Sha512};

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

#[must_use]
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

pub fn string_digest(input: &str, checksums: &mut [Checksum]) -> Option<Checksum> {
    if let Some(c) = checksums.iter().find(|&x| x.id == input) {
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

impl CsDigest for Sha256 {
    fn new_digest() -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self::new())
    }
    fn new_crc(_self: &mut Self) -> Option<&mut Self>
    where
        Self: Sized,
    {
        None
    }
    fn updated(&mut self, data: &[u8]) {
        self.update(data);
    }
    fn finalized(&mut self) -> Option<Box<[u8]>> {
        let f = Self::finalize(self.clone()).to_vec().into_boxed_slice();
        Some(f)
    }
}

impl CsDigest for Sha512 {
    fn new_digest() -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self::new())
    }
    fn new_crc(_self: &mut Self) -> Option<&mut Self>
    where
        Self: Sized,
    {
        None
    }
    fn updated(&mut self, data: &[u8]) {
        self.update(data);
    }
    fn finalized(&mut self) -> Option<Box<[u8]>> {
        let f = Self::finalize(self.clone()).to_vec().into_boxed_slice();
        Some(f)
    }
}

pub trait CsDigestB {
    fn updated(&mut self, data: &[u8]);
    fn finalized(&mut self) -> Option<Box<[u8]>>;
}

impl CsDigestB for crc::Digest<'_, u32> {
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

impl CsDigestB for crc::Digest<'_, u64> {
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

impl CsDigestB for crc::Digest<'_, u128> {
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
    #[must_use]
    pub fn new(id: &str, desc: &str, hash_type: HashType) -> Self {
        Self {
            id: id.to_string(),
            desc: desc.to_string(),
            hash_type,
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

impl Default for Checksums {
    fn default() -> Self {
        Self::new()
    }
}

impl Checksums {
    // new
    #[must_use]
    pub fn new() -> Self {
        Self {
            checksum: Checksum::new("sha", "Default Hash Sha-256", HashType::Sha),
            cache: BTreeMap::new(),
            max: 10000, // default,\
            use_hashmap: true,
        }
    }

    pub fn add(&mut self, hash: Box<[u8]>) -> Option<bool> {
        if self.cache.contains_key(&hash) {
            // exists
            Some(true)
        } else {
            if self.cache.len() > self.max {
                return None;
            }
            self.cache.insert(hash, true);
            Some(false)
        }
    }

    pub fn get_crc<T: CsDigestB>(digest: &mut T, data: &[u8]) -> Option<Box<[u8]>> {
        digest.updated(data);
        digest.finalized()
    }

    pub fn get_crc_blocks<T: CsDigestB>(digest: &mut T, data: &[Box<[u8]>]) -> Option<Box<[u8]>> {
        for block in data {
            digest.updated(block);
        }
        digest.finalized()
    }

    pub fn get_digest<T: CsDigest>(digest: &mut T, data: &[u8]) -> Option<Box<[u8]>> {
        digest.updated(data);
        digest.finalized()
    }

    #[must_use]
    pub fn digest_data(&self, data: &[u8]) -> Option<Box<[u8]>> {
        match &self.checksum.hash_type {
            HashType::Sha | HashType::Sha256 => {
                let mut d = Sha256::new_digest()?;
                Self::get_digest(&mut d, data)
            }
            HashType::Sha512 => {
                let mut d = Sha512::new_digest()?;
                Self::get_digest(&mut d, data)
            }
            HashType::Crc32 => {
                let cs = Crc::<u32>::new(&CRC_32_CKSUM);
                let mut d = cs.digest();
                Self::get_crc(&mut d, data)
            }
            HashType::Crc | HashType::Crc64 => {
                let cs = Crc::<u64>::new(&CRC_64_REDIS);
                let mut d = cs.digest();
                Self::get_crc(&mut d, data)
            }
            HashType::Crc82 => {
                let cs = Crc::<u128>::new(&CRC_82_DARC);
                let mut d = cs.digest();
                Self::get_crc(&mut d, data)
            }
        }
    }

    #[must_use]
    pub fn digest_blocks(&self, data: Option<&Vec<Box<[u8]>>>) -> Option<Box<[u8]>> {
        if let Some(data) = data {
            return match &self.checksum.hash_type {
                HashType::Sha | HashType::Sha256 | HashType::Sha512 => {
                    let digest: Option<Box<dyn CsDigest>> = match &self.checksum.hash_type {
                        HashType::Sha | HashType::Sha256 => {
                            let h = Sha256::new_digest()?;
                            Some(Box::new(h))
                        }
                        HashType::Sha512 => Some(Box::new(Sha512::new_digest()?)),
                        _ => None,
                    };
                    let mut d = digest?;
                    for block in data {
                        d.updated(block);
                    }
                    d.finalized()
                }
                HashType::Crc32 => {
                    let cs = Crc::<u32>::new(&CRC_32_CKSUM);
                    let mut d = cs.digest();
                    Self::get_crc_blocks(&mut d, data)
                }
                HashType::Crc | HashType::Crc64 => {
                    let cs = Crc::<u64>::new(&CRC_64_REDIS);
                    let mut d = cs.digest();
                    Self::get_crc_blocks(&mut d, data)
                }
                HashType::Crc82 => {
                    let cs = Crc::<u128>::new(&CRC_82_DARC);
                    let mut d = cs.digest();
                    Self::get_crc_blocks(&mut d, data)
                }
            };
        }
        None
    }
}
