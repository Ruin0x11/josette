use std::io::{self, Read, Write};
use std::mem;
use byteorder::{ByteOrder, BigEndian, WriteBytesExt};

pub trait Writeable {
    fn byte_size(&self) -> usize;
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error>;
}

impl Writeable for u8 {
    fn byte_size(&self) -> usize {
        mem::size_of::<u8>()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u8(*self)
    }
}

impl Writeable for u16 {
    fn byte_size(&self) -> usize {
        mem::size_of::<u16>()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u16::<BigEndian>(*self)
    }
}

impl Writeable for u32 {
    fn byte_size(&self) -> usize {
        mem::size_of::<u32>()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u32::<BigEndian>(*self)
    }
}

impl Writeable for usize {
    fn byte_size(&self) -> usize {
        mem::size_of::<usize>()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u32::<BigEndian>(*self as u32)
    }
}

impl<T: Writeable> Writeable for Vec<T> {
    fn byte_size(&self) -> usize {
        self.iter().map(|x| x.byte_size()).sum()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        for v in self.iter() {
            v.write(writer)?;
        }
        Ok(())
    }
}

impl Writeable for &str {
    fn byte_size(&self) -> usize {
        self.len()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_all(&self.as_bytes())
    }
}

impl Writeable for String {
    fn byte_size(&self) -> usize {
        let s: &str = &self;
        s.byte_size()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        let s: &str = &self;
        s.write(writer)
    }
}
