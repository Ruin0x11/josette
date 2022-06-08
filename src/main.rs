#[macro_use] extern crate anyhow;
extern crate nom;
extern crate hexyl;
extern crate tribool;

use nom::{ToUsize, IResult};
use nom::number::streaming::{be_u8, be_u32};
use nom::bytes::streaming::*;
use nom::multi::count;
use anyhow::{Context, Result};
use tribool::Tribool;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

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
    println!("data size: {}", data_size);
    let (input, data) = count(be_u8, data_size)(input)?;
    Ok((input, Spi { header, data }))
}

fn parse<T: AsRef<Path>>(filepath: T) -> Result<Spi> {
    let mut f = File::open(filepath.as_ref()).context("Unable to open file")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).context("Unable to read file")?;
    let (_, res) = spi(&buffer).map_err(|e| anyhow!("Parsing failed! {:?}", e))?;

    Ok(res)
}

fn printall(buffer: &[u8]) {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let mut printer = hexyl::Printer::new(
        &mut handle,
        true,
        false,
        true,
        hexyl::BorderStyle::Unicode,
        false,
    );

    printer.print_all(buffer).unwrap();
}

fn main() {
    let spi = parse("C:\\Users\\yuno\\Documents\\josette\\birdspi.bin").unwrap();

    let (s3, s1, s2) = spi.slices();

    let mut bit_no = 0;
    let mut s1_offset = 0;
    let mut s2_offset = 0;
    let mut s3_offset = 0;
    let mut pal_offset = 0x10;
    let mut is_other = false;

    let mut output = Vec::new();
    let mut palette = [0u32; 16];
    for i in 0..16 {
        palette[i] = i as u32;
    }

    assert!(spi.header.magic == "SPI1");

    printall(s1);
    printall(s2);
    printall(s3);

    let mut test_found = || {
        if bit_no == 8 {
            bit_no = 0;
            s3_offset += 1;
            if s3_offset >= s3.len() {
                return Tribool::Indeterminate;
            }
        }

        let shift = bit_no & 0x1f;
        bit_no += 1;
        println!("test! {} {} {}", s3[s3_offset] & 0x80, shift, s3[s3_offset] & 0x80 >> shift);
        if (s3[s3_offset] & 0x80 >> shift) != 0 {
            Tribool::True
        }
        else {
            Tribool::False
        }
    };

    while output.len() < spi.header.u1 {
        println!("header {} target {} output {} data {} {} {} {}", spi.header.u1, spi.header.u1, output.len(), spi.data.len(), s1.len(), s2.len(), s3.len());
        let found = test_found();
        if found.is_indeterminate() {
            println!("ranout");
            break;
        }

        if found.is_true() {
            let byte = match test_found() {
                Tribool::True => {
                    let it = s2[s2_offset];
                    s2_offset += 1;
                    let color_index = (pal_offset & 0xF) as usize;
                    palette[color_index] = it as u32;
                    pal_offset += 1;
                    println!("tt s2[{:02x}]={:02x} col={:02x}", s2_offset, it, color_index);
                    it
                },
                Tribool::False => {
                    println!("tf s1[{:02x}]={:02x}", s1_offset, s1[s1_offset]);
                    let color_index = if !is_other {
                        is_other = true;
                        (s1[s1_offset] >> 4) as usize
                    } else {
                        let thing = s1[s1_offset];
                        s1_offset += 1;
                        is_other = false;
                        (thing & 0xF) as usize
                    };

                    println!("tf col={:02x}", color_index);
                    palette[color_index] as u8
                },
                _ => unreachable!()
            };

            output.push(byte);
        }
        else {
            let mut a = (s2[s2_offset] as u32) >> 4;
            let b = s2[s2_offset] as usize;
            let c = s2[s2_offset + 1] as usize;
            println!("ff a b c {:02x} {:02x} {:02x}", a, b, c);
            s2_offset += 2;
            if a == 0xF {
                while s2[s2_offset] == 0xFF {
                    a += 0xFF;
                    s2_offset += 1;
                }
                let d = s2[s2_offset];
                s2_offset += 1;
                a += d as u32;
            }
            let mut pos = output.len() - c - 1;
            a += 3;
            println!("finala {:02x}", a);
            while a > 0 {
                a -= 1;
                // run-length encoding
                // TODO sized buffer
                println!("len {} b {:02x} c {:02x} write {:02x}", output.len(), b, c, output[pos]);
                output.push(output[pos]);
                pos += 1;
            }
        }
    }

    printall(&output[..]);
    let mut out = File::create("C:\\Users\\yuno\\Documents\\josette\\output.bin").unwrap();
    out.write_all(&output).unwrap();
}
