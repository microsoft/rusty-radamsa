//! Mux the outputs.
//!

use crate::generators::GenericReader;
use crate::shared::*;
use log::*;
use std::fs::File;
use std::io::{self, Cursor};
use std::net::{TcpStream, UdpSocket};

#[cfg(not(test))]
use log::debug;

#[cfg(test)]
use std::println as debug;

pub const DEFAULT_OUTPUTS: &'static str = "-";

pub fn init_outputs() -> Vec<Output> {
    Vec::from([
        Output::new("-","Write output data to Stdout", OutputType::Stdout),
        Output::new("file", "Write output data to a binary file", OutputType::File),
        Output::new("tcpserver", "Write output data to a tcp port as server",OutputType::TCPServer), 
        Output::new("tcpclient", "Write output data to a tcp port as client",OutputType::TCPClient),
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
impl Outputs {
    pub fn new() -> Outputs {
        Outputs {
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
        _buffer: &Option<&mut Box<[u8]>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut new_outputs: Vec<Output> = vec![];
        for output in &self.outputs {
            if let Some(paths) = &output.paths {
                if 0 < paths.len() {
                    for p in paths {
                        let mut new_output = output.clone();
                        if new_output.set_fd(Some(p.clone()), &None).is_ok() {
                            new_outputs.push(new_output);
                        }
                    }
                } else {
                    let mut new_output = output.clone();
                    if new_output.set_fd(None, &None).is_ok() {
                        new_outputs.push(new_output);
                    }
                }
            } else {
                let mut new_output = output.clone();
                if new_output.set_fd(None, _buffer).is_ok() {
                    new_outputs.push(new_output);
                }
            }
        }
        self.outputs = new_outputs;
        Ok(())
    }
    pub fn mux_output(
        &mut self,
        _data: &Vec<u8>,
        _buffer: &mut Option<&mut Box<[u8]>>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        debug!("mux output");
        let data: Vec<u8> = match self.truncate {
            0 => _data.clone(), // if truncate is zero, no truncation happens
            _ => {
                if self.truncate > _data.len() {
                    _data.clone()
                } else {
                    _data[..self.truncate].to_vec()
                }
            }
        };
        for output in &mut self.outputs {
            debug!("writing to {}", output.id);
            output.write(&data)?;
            if output.fd_type == OutputType::Buffer {
                if let Some(ref mut buf) = _buffer.as_mut() {
                    if self.resize {
                        let resize_len = match self.truncate {
                            0 => data.len(),
                            _ => self.truncate,
                        };
                        let vec = vec![0u8; resize_len];
                        ***buf = vec.into_boxed_slice();
                        buf[..resize_len].clone_from_slice(&data[..resize_len]);
                    } else {
                        let mut max_len = buf.len();
                        if data.len() < buf.len() {
                            max_len = data.len();
                        }
                        let gr: &dyn crate::generators::GenericReader =
                            output.fd.as_ref().unwrap().as_ref();
                        let cursor: &Cursor<Box<[u8]>> = gr
                            .as_any()
                            .downcast_ref::<Cursor<Box<[u8]>>>()
                            .expect("Wasn't a trusty printer!");
                        let vec = cursor.get_ref();
                        buf[..max_len].clone_from_slice(&vec[..max_len]);
                    }
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
    fn clone(&self) -> Output {
        Output {
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
            .finish()
    }
}

impl Output {
    pub fn new(_id: &str, _desc: &str, _type: OutputType) -> Output {
        Output {
            id: _id.to_string(),
            desc: _desc.to_string(),
            fd_type: _type,
            fd: None,
            paths: None,
        }
    }
    pub fn set_fd(
        &mut self,
        _path: Option<String>,
        _buf: &Option<&mut Box<[u8]>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // initialize the fd
        let fd = get_fd(&self.fd_type, _path, _buf)?;
        self.fd = Some(fd);
        Ok(())
    }
    pub fn write(&mut self, _data: &Vec<u8>) -> Result<usize, Box<dyn std::error::Error>> {
        match self.fd {
            Some(ref mut fd) => fd.gen_write(_data, 0),
            None => {
                error!("fd failed for {}", self.id);
                Ok(0)
            }
        }
    }
    pub fn flush_bvecs(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        match self.fd {
            Some(ref mut fd) => fd.gen_flush(),
            None => {
                error!("fd failed for {}", self.id);
                Ok(0)
            }
        }
    }
    pub fn write_all(&mut self, _data: &Vec<Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> {
        match self.fd {
            Some(ref mut fd) => {
                for d in _data {
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

pub fn string_outputs(_input: Vec<&str>, _outputs: &mut Vec<Output>) -> Vec<Output> {
    debug!("string_outputs");
    let mut applied_outputs: Vec<Output> = vec![];
    if _input.is_empty() {
        return vec![];
    }
    debug!("_input {:?}", _input);
    let mut iter = _input.iter().peekable();
    debug!("_outputs {:?}", iter);
    while let Some(next) = iter.next() {
        debug!("o {:?}", next);
        if let Some(o) = _outputs.iter().find(|&x| x.id.eq(next)) {
            debug!("o {:?}", o);
            match o.fd_type {
                OutputType::Buffer | OutputType::Stdout => {
                    applied_outputs.push(o.clone());
                    continue;
                }
                _ => {
                    let mut paths: Vec<String> = Vec::new();
                    while let Some(path) = iter.next() {
                        paths.push(path.to_string());
                        if let Some(peek) = iter.peek() {
                            if let Some(_) = _outputs.iter().find(|&x| x.id.eq(*peek)) {
                                break;
                            }
                        }
                    }
                    let mut output = o.clone();
                    output.paths = Some(paths);
                    applied_outputs.push(output.clone());
                }
            }
        }
    }
    debug!("applied_outputs {:?}", applied_outputs);
    applied_outputs
}

pub fn get_fd(
    _type: &OutputType,
    _path: Option<String>,
    _buf: &Option<&mut Box<[u8]>>,
) -> Result<Box<dyn GenericReader>, Box<dyn std::error::Error>> {
    match *_type {
        OutputType::Stdout => Ok(Box::new(io::Stdout::gen_open("w", None, None)?)),
        OutputType::File => Ok(Box::new(File::gen_open("w", _path, None)?)),
        OutputType::TCPServer => Ok(Box::new(TcpStream::gen_open("w", _path, None)?)),
        OutputType::TCPClient => Ok(Box::new(TcpStream::gen_open("w", _path, None)?)),
        OutputType::UDPServer => Ok(Box::new(UdpSocket::gen_open("w", _path, None)?)),
        OutputType::UDPClient => Ok(Box::new(UdpSocket::gen_open("w", _path, None)?)),
        OutputType::Buffer => {
            if let Some(ref buf) = _buf {
                let b: Box<[u8]> = (**buf).to_owned();
                Ok(Box::new(Cursor::<Box<[u8]>>::gen_open("w", None, Some(b))?))
            } else {
                Err(Box::new(NoneString))
            }
        }
        OutputType::Hashing => Err(Box::new(NoneString)),
        OutputType::Template => Err(Box::new(NoneString)),
    }
}
