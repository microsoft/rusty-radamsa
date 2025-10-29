//! Mux the outputs.
//!

use std::fs::File;
use std::io::{self, Cursor};
use std::net::{TcpStream, UdpSocket};
#[cfg(test)]
use std::println as debug;

#[cfg(not(test))]
use log::debug;
use log::*;

use crate::generators::GenericReader;
use crate::shared::*;

pub const DEFAULT_OUTPUTS: &str = "-";

#[must_use]
pub fn init_outputs() -> Vec<Output> {
    Vec::from([
        Output::new("-", "Write output data to Stdout", OutputType::Stdout),
        Output::new("file", "Write output data to a binary file", OutputType::File),
        Output::new("tcpserver", "Write output data to a tcp port as server", OutputType::TCPServer),
        Output::new("tcpclient", "Write output data to a tcp port as client", OutputType::TCPClient),
        Output::new("udpserver", "Write output data to a udp port as server", OutputType::UDPServer),
        Output::new("udpclient", "Write output data to a udp port as client", OutputType::UDPClient),
        Output::new("buffer", "Write output data to a buffer address or vector", OutputType::Buffer),
        Output::new("hash", "Write output variations or a hashing directory using %n and %s as in the template path (i.e. /tmp/fuzz-%n.%s)", OutputType::Hashing),
        Output::new("template", "Output template. %f is fuzzed data. e.g. \"<html>%f</html>\"", OutputType::Template),
    ])
}

#[derive(Debug)]
pub struct Outputs {
    pub outputs: Vec<Output>,
    pub truncate: usize,
    pub resize: bool,
}

#[allow(clippy::new_without_default)]
impl Outputs {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            outputs: Vec::new(),
            truncate: 0,
            resize: false,
        }
    }

    pub fn init(&mut self) {
        self.outputs = init_outputs();
    }

    pub fn default_outputs(&mut self) {
        self.outputs = string_outputs(vec!["buffer", DEFAULT_OUTPUTS], &mut self.outputs);
    }

    pub fn init_pipes(
        &mut self,
        buffer: &Option<&mut Box<[u8]>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut new_outputs: Vec<Output> = vec![];
        for output in &self.outputs {
            if let Some(paths) = &output.paths {
                if paths.is_empty() {
                    let mut new_output = output.clone();
                    if new_output.set_fd(None, &None).is_ok() {
                        new_outputs.push(new_output);
                    }
                } else {
                    for p in paths {
                        let mut new_output = output.clone();
                        if new_output.set_fd(Some(p.clone()), &None).is_ok() {
                            new_outputs.push(new_output);
                        }
                    }
                }
            } else {
                let mut new_output = output.clone();
                if new_output.set_fd(None, buffer).is_ok() {
                    new_outputs.push(new_output);
                }
            }
        }
        self.outputs = new_outputs;
        Ok(())
    }

    pub fn mux_output(
        &mut self,
        data: &[u8],
        buffer: &mut Option<&mut Box<[u8]>>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        debug!("mux output");
        let data: Vec<u8> = match self.truncate {
            0 => data.to_owned(), // if truncate is zero, no truncation happens
            _ => {
                if self.truncate > data.len() {
                    data.to_owned()
                } else {
                    data[..self.truncate].to_vec()
                }
            }
        };
        for output in &mut self.outputs {
            debug!("writing to {}", output.id);
            output.write(&data)?;
            if output.fd_type == OutputType::Buffer
                && let Some(ref mut buf) = buffer.as_mut() {
                if self.resize {
                    let resize_len = match self.truncate {
                        0 => data.len(),
                        _ => self.truncate,
                    };
                    let vec = vec![0u8; resize_len];
                    ***buf = vec.into_boxed_slice();
                    buf[..resize_len].clone_from_slice(&data[..resize_len]);
                } else {
                    let max_len = if data.len() < buf.len() {
                        data.len()
                    } else {
                        buf.len()
                    };
                    let gr: &dyn GenericReader = output.fd.as_ref().unwrap().as_ref();
                    let cursor: &Cursor<Box<[u8]>> = gr
                        .as_any()
                        .downcast_ref::<Cursor<Box<[u8]>>>()
                        .expect("Wasn't a trusty printer!");
                    let vec = cursor.get_ref();
                    buf[..max_len].clone_from_slice(&vec[..max_len]);
                }
            }
            output.flush_bvecs()?;
        }
        Ok(data.len())
    }
}

pub struct Output {
    pub id: String,
    pub desc: String,
    pub fd_type: OutputType,
    pub fd: Option<Box<dyn GenericReader>>,
    pub paths: Option<Vec<String>>,
}

impl Clone for Output {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            desc: self.desc.clone(),
            fd_type: self.fd_type,
            fd: None,
            paths: self.paths.clone(),
        }
    }
}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Output")
            .field("id", &self.id)
            .field("desc", &self.desc)
            .field("fd_type", &self.fd_type)
            .field("paths", &self.paths)
            .finish_non_exhaustive()
    }
}

impl Output {
    #[must_use]
    pub fn new(id: &str, desc: &str, output_type: OutputType) -> Self {
        Self {
            id: id.to_string(),
            desc: desc.to_string(),
            fd_type: output_type,
            fd: None,
            paths: None,
        }
    }

    pub fn set_fd(
        &mut self,
        path: Option<String>,
        buf: &Option<&mut Box<[u8]>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // initialize the fd
        let fd = get_fd(&self.fd_type, path, buf)?;
        self.fd = Some(fd);
        Ok(())
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        if let Some(ref mut fd) = self.fd {
            fd.gen_write(data, 0)
        } else {
            error!("fd failed for {}", self.id);
            Ok(0)
        }
    }

    pub fn flush_bvecs(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        if let Some(ref mut fd) = self.fd {
            fd.gen_flush()
        } else {
            error!("fd failed for {}", self.id);
            Ok(0)
        }
    }

    pub fn write_all(&mut self, data: &Vec<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
        match self.fd {
            Some(ref mut fd) => {
                for d in data {
                    fd.gen_write(d, 0)?;
                }
            }
            None => {
                error!("fd failed for {}", self.id);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    Stdout,
    File,
    TCPServer,
    TCPClient,
    UDPServer,
    UDPClient,
    Buffer,
    Hashing,
    Template,
}

#[allow(clippy::needless_pass_by_value)]
pub fn string_outputs(input: Vec<&str>, outputs: &mut [Output]) -> Vec<Output> {
    debug!("string_outputs");
    let mut applied_outputs: Vec<Output> = vec![];
    if input.is_empty() {
        return vec![];
    }
    debug!("_input {input:?}");
    let mut iter = input.iter().peekable();
    debug!("_outputs {iter:?}",);
    while let Some(next) = iter.next() {
        debug!("o {next:?}");
        if let Some(o) = outputs.iter().find(|&x| x.id.eq(next)) {
            debug!("o {o:?}");
            match o.fd_type {
                OutputType::Buffer | OutputType::Stdout => {
                    applied_outputs.push(o.clone());
                }
                _ => {
                    let mut paths: Vec<String> = Vec::new();
                    while let Some(path) = iter.next() {
                        paths.push((*path).to_string());
                        if let Some(peek) = iter.peek() &&
                            outputs.iter().any(|x| x.id.eq(*peek)) {
                            break;
                        }
                    }
                    let mut output = o.clone();
                    output.paths = Some(paths);
                    applied_outputs.push(output.clone());
                }
            }
        }
    }
    debug!("applied_outputs {applied_outputs:?}");
    applied_outputs
}

pub fn get_fd(
    output_type: &OutputType,
    path: Option<String>,
    buf: &Option<&mut Box<[u8]>>,
) -> Result<Box<dyn GenericReader>, Box<dyn std::error::Error>> {
    match *output_type {
        OutputType::Stdout => Ok(Box::new(io::Stdout::gen_open("w", None, None)?)),
        OutputType::File => Ok(Box::new(File::gen_open("w", path, None)?)),
        OutputType::TCPServer | OutputType::TCPClient => {
            Ok(Box::new(TcpStream::gen_open("w", path, None)?))
        }
        OutputType::UDPServer | OutputType::UDPClient => {
            Ok(Box::new(UdpSocket::gen_open("w", path, None)?))
        }
        OutputType::Buffer => {
            if let Some(buf) = buf {
                let b: Box<[u8]> = (**buf).clone();
                Ok(Box::new(Cursor::<Box<[u8]>>::gen_open("w", None, Some(b))?))
            } else {
                Err(Box::new(NoneString))
            }
        }
        OutputType::Hashing | OutputType::Template => Err(Box::new(NoneString)),
    }
}
