use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take, take_while};
use nom::character::complete::char;
use nom::combinator::{consumed, verify};
use nom::multi::{fold_many0, many0};
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::{IResult, Parser};
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

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Delimited {
    pub delimitor: u8,
    pub data: Vec<u8>,
}

impl Delimited {
    fn delimitor(c: char, data: &[u8]) -> Self {
        Self {
            delimitor: c as u8,
            data: data.to_owned(),
        }
    }

    fn unlex(self, v: &mut Vec<u8>) {
        v.push(self.delimitor);
        v.extend(self.data.into_iter());
        v.push(self.delimitor);
    }
}

/// parse a string delimited by quotes
fn parse_quoted_string(delim: char) -> impl FnMut(&[u8]) -> IResult<&[u8], Delimited> {
    move |input: &[u8]| {
        // parse until a terminating quote character, ignoring escaped quotes
        let build_string = consumed(many0(alt((
            // parse until a delim or an escape char
            verify(
                take_while(|c: u8| c != '\\' as u8 && c != delim as u8),
                |s: &[u8]| !s.is_empty(),
            ),
            // eat escaped literals
            preceded(char('\\'), take(1usize)),
        ))))
        .map(|(consumed, _)| consumed);

        // parse the entire quote-delimited string
        delimited(char(delim), build_string, char(delim))
            .map(|data| Delimited::delimitor(delim, data))
            .parse(input)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Text {
    Texty(Vec<u8>),
    Delim(Delimited),
}

impl Text {
    fn texty(s: &[u8]) -> Self {
        Self::Texty(s.to_owned())
    }

    fn mutate(&mut self, _rng: &mut dyn RngCore) {
        match self {
            Self::Texty(d) => mutate_text_data(_rng, d),
            Self::Delim(d) => mutate_text_data(_rng, &mut d.data),
        }
    }

    fn unlex(self, v: &mut Vec<u8>) {
        match self {
            Text::Texty(a) => v.extend(a.into_iter()),
            Text::Delim(delim) => delim.unlex(v),
        }
    }
}

fn parse_texty<'a>(input: &'a [u8]) -> IResult<&'a [u8], Vec<Text>> {
    let fold_ascii = |mut acc: Vec<Text>, dat: Text| {
        match (acc.last_mut(), dat) {
            // coalesce contiguous texty blocks
            (Some(Text::Texty(ref mut a)), Text::Texty(b)) => a.extend(b.into_iter()),
            (_, dat) => acc.push(dat),
        }
        acc
    };

    let parse_single_quote = alt((
        parse_quoted_string('\'').map(Text::Delim),
        // if coule not parse full delimited string, skip
        tag(b"'").map(Text::texty),
    ));

    let parse_double_quote = alt((
        parse_quoted_string('"').map(Text::Delim),
        // if coule not parse full delimited string, skip
        tag(b"\"").map(Text::texty),
    ));

    fold_many0(
        alt((
            verify(is_not(r#"'""#), |x: &[u8]| !x.is_empty()).map(Text::texty),
            parse_single_quote,
            parse_double_quote,
        )),
        Vec::new,
        fold_ascii,
    )
    .parse(input)
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Data {
    Bytes(Vec<u8>),
    Texty(Vec<Text>),
}

impl Data {
    fn unlex(self, v: &mut Vec<u8>) {
        match self {
            Data::Texty(a) => {
                for i in a.into_iter() {
                    i.unlex(v);
                }
            }
            Data::Bytes(a) => v.extend(a.into_iter()),
        }
    }
}

fn is_texty(x: u8) -> bool {
    match x {
        9 | 10 | 13 | 31..=125 => true,
        _ => false,
    }
}

fn parse_texty_bytes<'a>(min_texty: usize) -> impl FnMut(&[u8]) -> IResult<&[u8], Vec<Text>> {
    move |input: &[u8]| {
        // returns success if more than min_texty contiguous "texty" bytes found
        let texty = verify(take_while(is_texty), |x: &[u8]| x.len() >= min_texty);
        texty.and_then(parse_texty).parse(input)
    }
}

fn parse_bytes<'a>(min_texty: usize) -> impl FnMut(&[u8]) -> IResult<&[u8], Vec<Data>> {
    let fold_bytes = |mut acc: Vec<Data>, dat: Data| {
        match (acc.last_mut(), dat) {
            // coalesce contiguous byte blocks
            (Some(Data::Bytes(ref mut a)), Data::Bytes(b)) => a.extend(b.into_iter()),
            (_, dat) => acc.push(dat),
        }
        acc
    };

    move |input: &[u8]| {
        // require texty bytes at the beginning of the string
        let (remaining, (text, rest)) = parse_texty_bytes(min_texty)
            .and(fold_many0(
                alt((
                    // if we had enough texty bytes, try parsing it
                    parse_texty_bytes(min_texty).map(Data::Texty),
                    // otherwise record it as a byte and pass through
                    take(1usize).map(|x: &[u8]| Data::Bytes(x.to_owned())),
                )),
                Vec::new,
                fold_bytes,
            ))
            .parse(input)?;

        assert_eq!(
            remaining, b"",
            "If parsing succeeded, all input was consumed"
        );

        // combine the mandatory first text chunk with rest of the data
        let mut ret = vec![Data::Texty(text.to_owned())];
        ret.extend(rest.into_iter());
        Ok((remaining, ret))
    }
}

pub struct Ascii(Vec<Data>);

impl Ascii {
    pub(crate) fn parse(data: &[u8]) -> Result<Self, ()> {
        if let Ok((_, ret)) = parse_bytes(6).parse(data) {
            Ok(Self(ret))
        } else {
            Err(())
        }
    }

    pub(crate) fn mutate(&mut self, _rng: &mut dyn RngCore) {
        loop {
            // find a mutatable chunk, ignoring non-ascii data
            if let Data::Texty(ref mut dat) = self.0.choose_mut(_rng).unwrap() {
                dat.choose_mut(_rng).unwrap().mutate(_rng);
                break;
            }
        }
    }

    pub(crate) fn unlex(self) -> Vec<u8> {
        let mut ret = Vec::new();
        for i in self.0.into_iter() {
            i.unlex(&mut ret);
        }
        ret
    }
}

#[cfg(test)]
mod ascii_bad {
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    use crate::mutations::ascii::{Data, Delimited, Text};

    use super::Ascii;

    #[test]
    fn basic() {
        // $ printf "AAAAAA\x00\x01\x02AAAAAA" | ./bin/ol -r ./rad/main.scm --mutations "ab" --patterns "od"
        let data = b"AAAAAA\x00\x01\x02AAAAAA".to_vec();
        let cs = Ascii::parse(&data).unwrap();
        assert_eq!(cs.0.len(), 3);
        assert_eq!(cs.0[0], Data::Texty(vec![Text::texty(b"AAAAAA")]));
        assert_eq!(cs.0[1], Data::Bytes(vec![0x00, 0x01, 0x02]));
        assert_eq!(cs.0[2], Data::Texty(vec![Text::texty(b"AAAAAA")]));
        let data_unlex: Vec<u8> = cs.unlex();
        assert_eq!(data, data_unlex);
    }

    #[test]
    fn delim() {
        // $ printf "AAAAAA\x00\x01\x02'AAAAAA'" | ./bin/ol -r ./rad/main.scm --mutations "ab" --patterns "od"
        let data = b"AAAAAA\x00\x01\x02'AAAAAA'".to_vec();
        let cs = Ascii::parse(&data).unwrap();
        assert_eq!(cs.0.len(), 3);
        assert_eq!(cs.0[0], Data::Texty(vec![Text::texty(b"AAAAAA")]));
        assert_eq!(cs.0[1], Data::Bytes(vec![0x00, 0x01, 0x02]));
        assert_eq!(
            cs.0[2],
            Data::Texty(vec![Text::Delim(Delimited::delimitor('\'', b"AAAAAA"))])
        );
        let data_unlex: Vec<u8> = cs.unlex();
        assert_eq!(data, data_unlex);
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
            if let Ok(cs) = Ascii::parse(&data) {
                assert_eq!(&data, &cs.unlex());
            }
        }
    }

    #[test]
    fn first_chunk_byte() {
        // if the first chunk is not text, Ascii should fail to parse
        let data = b"\x01AAAAAAAAAAAAAAAA";
        let res = Ascii::parse(data);
        assert!(res.is_err());
    }
}
