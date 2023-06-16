use rand::{seq::SliceRandom, Rng, RngCore};

lazy_static! {
    static ref SILLY_STRINGS: Vec<Vec<u8>> = {
        // XXX: extend this because many of these strings are incredibly Linux-specific
        // perhaps add a configurable wordlist
        #[rustfmt::skip]
        let ret = vec![
            "%n", "%n", "%s", "%d", "%p", "%#x",
            "\\00", "aaaa%d%n",
            "`xcalc`", ";xcalc", "$(xcalc)", "!xcalc", "\"xcalc", "'xcalc",
            "\\x00", "\\r\\n", "\\r", "\\n", "\\x0a", "\\x0d",
            "NaN", "+inf",
            "$PATH",
            "$!!", "!!", "&#000;", "\\u0000",
            "$&", "$+", "$`", "$'", "$1",
        ];
        ret.into_iter().map(|x| x.as_bytes().to_owned()).collect()
    };
}

fn random_badness(_rng: &mut dyn RngCore) -> Vec<u8> {
    // concatenate between 1 and 20 random silly strings
    let mut v = Vec::new();
    for _ in 0.._rng.gen_range(1..20) {
        v.extend(SILLY_STRINGS.choose(_rng).unwrap());
    }
    v
}

fn mutate_text_data(_rng: &mut dyn RngCore, data: &mut Vec<u8>) {
    assert!(data.len() > 0);
    let idx = _rng.gen_range(0..data.len());
    match _rng.gen_range(0..=2) {
        0 => {
            // insert badness
            let badness = random_badness(_rng);
            for (i, d) in badness.into_iter().enumerate() {
                data.insert(idx + i, d);
            }
        }
        1 => {
            // replace badness
            let badness = random_badness(_rng);
            data.truncate(idx);
            data.extend(&badness);
        }
        2 => {
            // push random number of newline characters
            let num_as = match _rng.gen_range(0..=10) {
                0 => 127,
                1 => 128,
                2 => 255,
                3 => 256,
                4 => 16383,
                5 => 16384,
                6 => 32767,
                7 => 32768,
                8 => 65535,
                9 => 65536,
                _ => _rng.gen_range(0..1024),
            };
            for i in 0..num_as {
                data.insert(idx + i, 0xa);
            }
        }
        _ => unreachable!(),
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Delimiter {
    SingleQuote,
    DoubleQuote,
}

impl Delimiter {
    pub(crate) fn as_u8(&self) -> u8 {
        match self {
            Self::DoubleQuote => 34,
            Self::SingleQuote => 39,
        }
    }
    pub(crate) fn try_from(x: u8) -> Option<Self> {
        if x == 34 {
            return Some(Self::DoubleQuote);
        }
        if x == 39 {
            return Some(Self::SingleQuote);
        }
        return None;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Lex {
    Text(Vec<u8>),
    Byte(Vec<u8>),
    Delimited(Delimiter, Vec<u8>),
}

fn is_texty(x: &u8) -> bool {
    match x {
        9 | 10 | 13 | 31..=125 => true,
        _ => false,
    }
}

fn is_texty_enough(data: &[u8]) -> bool {
    let min_texty = 6;
    data.iter().take(min_texty).all(is_texty)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Ascii(Vec<Lex>);

impl Ascii {
    // XXX: original string-lex function would identify delimiters (quotes or double quotes). Punt for now. Perhaps improve on radamsa's rudimentary algorithms
    pub(crate) fn lex(data: &[u8]) -> Ascii {
        let mut chunks = Vec::new();

        let mut seen_data = Vec::new();
        let mut i = 0;
        while i < data.len() {
            if is_texty_enough(&data[i..]) {
                if seen_data.len() > 0 {
                    // "flush" any raw bytes
                    chunks.push(Lex::Byte(seen_data.clone()));
                    seen_data.clear();
                }
                let mut seen_text = Vec::new();
                while i < data.len() {
                    if !is_texty(&data[i]) {
                        break;
                    }
                    seen_text.push(data[i]);
                    i += 1;
                }
                chunks.push(Lex::Text(seen_text))
            } else {
                seen_data.push(data[i]);
                i += 1;
            }
        }
        if seen_data.len() > 0 {
            // "flush" any raw bytes
            chunks.push(Lex::Byte(seen_data.clone()));
        }

        Self(chunks)
    }

    pub(crate) fn first_block_has_text(&self) -> bool {
        match self.0.get(0) {
            None | Some(Lex::Byte(_)) => false,
            Some(_) => true,
        }
    }

    pub(crate) fn mutate(&mut self, _rng: &mut dyn RngCore) {
        loop {
            // find a mutatable chunk, ignoring non-ascii data
            match self.0.choose_mut(_rng).unwrap() {
                Lex::Text(ref mut dat) | Lex::Delimited(_, ref mut dat) => {
                    mutate_text_data(_rng, dat);
                    break;
                }
                Lex::Byte(_) => {}
            }
        }
    }

    pub(crate) fn unlex(self) -> Vec<u8> {
        let mut ret = Vec::new();
        for i in self.0 {
            match i {
                Lex::Byte(a) => ret.extend(a),
                Lex::Text(a) => ret.extend(a),
                Lex::Delimited(d, s) => {
                    ret.push(d.as_u8());
                    ret.extend(s);
                    ret.push(d.as_u8());
                }
            }
        }
        ret
    }
}

#[cfg(test)]
mod ascii_bad {
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    use super::{Ascii, Delimiter, Lex};

    #[test]
    fn basic() {
        // $ printf "AAAAAA\x00\x01\x02AAAAAA" | ./bin/ol -r ./rad/main.scm --mutations "ab" --patterns "od"
        let data = b"AAAAAA\x00\x01\x02AAAAAA".to_vec();
        let cs = Ascii::lex(&data);
        let a0 = Lex::Text(vec![0x41; 6]);
        let a1 = Lex::Byte(vec![0x00, 0x01, 0x02]);
        let a2 = Lex::Text(vec![0x41; 6]);
        assert_eq!(cs.0.len(), 3);
        assert_eq!(cs.0[0], a0);
        assert_eq!(cs.0[1], a1);
        assert_eq!(cs.0[2], a2);
        let data_unlex: Vec<u8> = cs.unlex();
        assert_eq!(data, data_unlex);
    }

    // expect this to fail since we haven't implemented delimiter checking yet
    #[test]
    fn delim() {
        // $ printf "AAAAAA\x00\x01\x02'AAAAAA'" | ./bin/ol -r ./rad/main.scm --mutations "ab" --patterns "od"
        let data = b"AAAAAA\x00\x01\x02'AAAAAA'".to_vec();
        let cs = Ascii::lex(&data);
        let a0 = Lex::Text(vec![0x41; 6]);
        let a1 = Lex::Byte(vec![0x00, 0x01, 0x02]);
        let a2 = Lex::Delimited(Delimiter::SingleQuote, vec![0x41; 6]);
        assert_eq!(cs.0.len(), 3);
        assert_eq!(cs.0[0], a0);
        assert_eq!(cs.0[1], a1);
        assert_ne!(cs.0[2], a2); // expect to fail
        let data_unlex = cs.unlex();
        assert_eq!(data, data_unlex);
    }

    #[test]
    fn mutate_smoke_test() {
        let mut rng = ChaCha20Rng::seed_from_u64(1683310580);
        for _ in 0..1000 {
            let a0 = Lex::Text(vec![0x41; 6]);
            let a1 = Lex::Byte(vec![0x00, 0x01, 0x02]);
            let a2 = Lex::Delimited(Delimiter::SingleQuote, vec![0x41; 6]);
            let mut ascii = Ascii(vec![a0, a1, a2]);
            for _ in 0..10 {
                ascii.mutate(&mut rng);
            }
        }
    }

    #[test]
    fn lex_roundtrip_smoke_test() {
        let mut rng = ChaCha20Rng::seed_from_u64(1683310580);
        let mut data = vec![0u8; 1000];
        for _ in 0..1000 {
            // generate random buffer
            for i in 0..1000 {
                data[i] = rng.gen();
            }
            let cs = Ascii::lex(&data);
            let roundtrip = cs.unlex();
            assert_eq!(&data, &roundtrip);
        }
    }
}
