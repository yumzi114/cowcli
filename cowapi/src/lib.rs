
use std::{borrow::BorrowMut, io::{Cursor, ErrorKind,Error}, ops::Index, sync::{Arc, Mutex}};
use bytes::{BytesMut, BufMut};
use tokio_util::codec::{Decoder, Encoder};
use std::{fmt::Debug, io, str, u8};
use std::io::Write;

#[cfg(unix)]
// const DEFAULT_TTY: &str = env!("SERIAL_DEVICE");
const DEFAULT_TTY: &str = "/dev/ttyACM0";

#[derive(Clone,Copy)]
pub struct LineCodec ;

impl Decoder for LineCodec {
    type Item = Vec<u8>;
    type Error = io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, std::io::Error> {
        let start = src.as_ref().iter().position(|x| *x == 0xFC);
        // let start = src.as_ref().iter().position(|x| *x == b'\n');
        if let Some(n) = start {
            let line = src.split_to(n+1);
            if line.len()==13&&line[0]==0xAF&&line[line.len()-1]==0xFC{
                let line_list = line.to_vec();
                return Ok(Some(line_list));
            }
        }
        // if let Some(n) = start {
        //     let line = src.split_to(n+1);
        //     let line_list = line.to_vec();
        //     return Ok(Some(line_list));
        // }
 
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;
    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}


