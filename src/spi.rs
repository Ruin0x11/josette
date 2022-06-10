use std::io::{self, Read, Write};
use nom::{ToUsize, IResult};
use nom::number::streaming::{be_u8, be_u32};
use nom::bytes::streaming::*;
use nom::multi::count;
use crate::writeable::Writeable;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SpiHeader {
    pub magic: String,
    pub u1: usize,
    pub u2: usize,
    pub u3: usize,
    pub u4: usize,
}

pub fn spi_header(input: &[u8]) -> IResult<&[u8], SpiHeader> {
    let (input, magic) = take(4u8)(input)?;
    let (input, u1) = be_u32(input).map(|(i, u)| (i, u.to_usize()))?;
    let (input, u2) = be_u32(input).map(|(i, u)| (i, u.to_usize()))?;
    let (input, u3) = be_u32(input).map(|(i, u)| (i, u.to_usize()))?;
    let (input, u4) = be_u32(input).map(|(i, u)| (i, u.to_usize()))?;
    Ok((input, SpiHeader { magic: std::str::from_utf8(magic).unwrap().to_string(), u1, u2, u3, u4 }))
}

impl Writeable for SpiHeader {
    fn byte_size(&self) -> usize {
        self.magic.byte_size()
            + self.u1.byte_size()
            + self.u2.byte_size()
            + self.u3.byte_size()
            + self.u4.byte_size()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.magic.write(writer)?;
        self.u1.write(writer)?;
        self.u2.write(writer)?;
        self.u3.write(writer)?;
        self.u4.write(writer)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Spi {
    pub header: SpiHeader,
    pub data: Vec<u8>
}

impl Spi {
    pub fn slices(&self) -> (&[u8], &[u8], &[u8]) {
        let u2 = self.header.u2;
        let u3 = self.header.u3;
        let u4 = self.header.u4;
        (&self.data[0..u2], &self.data[u2..u2+u3], &self.data[u2+u3..u2+u3+u4])
    }
}

pub fn spi(input: &[u8]) -> IResult<&[u8], Spi>{
    let (input, header) = spi_header(input)?;
    let data_size = header.u2 + header.u3 + header.u4;
    // println!("data size: {}", data_size);
    let (input, data) = count(be_u8, data_size)(input)?;
    Ok((input, Spi { header, data }))
}

impl Writeable for Spi {
    fn byte_size(&self) -> usize {
        self.header.byte_size()
            + self.data.byte_size()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.header.write(writer)?;
        self.data.write(writer)
    }
}
