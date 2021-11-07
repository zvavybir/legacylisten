use std::{
    cmp::min,
    io::{self, Read, Seek},
};

use crate::Error;

pub struct Buffer
{
    buf: Vec<u8>,
    pos: usize,
}

impl Read for Buffer
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error>
    {
        if self.pos >= self.buf.len()
        {
            // TODO: Return correct error
            return Ok(0);
        }
        let slice = &self.buf[self.pos..];
        let slice = &slice[..min(buf.len(), slice.len())];
        let len = min(slice.len(), buf.len());

        (buf[..len]).copy_from_slice(&slice[..len]);

        self.pos += len;

        Ok(len)
    }
}

impl Seek for Buffer
{
    fn seek(&mut self, pos: io::SeekFrom) -> Result<u64, io::Error>
    {
        match pos
        {
            io::SeekFrom::Current(x) => self.pos += x as usize,
            io::SeekFrom::Start(x) => self.pos = x as usize,
            io::SeekFrom::End(x) =>
            {
                let x = x as usize;
                if self.buf.len() + x == 0
                {
                    // TODO: Return `io::Error` with `ErrorKind` being
                    // `InvalidInput`.
                    self.pos = 0;
                }
                else
                {
                    self.pos = self.buf.len() + x - 1;
                }
            }
        }

        Ok(self.pos as _)
    }
}

impl Buffer
{
    pub fn new<R: Read>(mut reader: R) -> Result<Self, Error>
    {
        let mut buf = Self {
            buf: vec![],
            pos: 0,
        };
        reader.read_to_end(&mut buf.buf)?;

        Ok(buf)
    }
}
