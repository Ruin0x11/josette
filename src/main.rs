#[macro_use] extern crate anyhow;
extern crate nom;
extern crate hexyl;
extern crate tribool;
extern crate rgb;
#[macro_use] extern crate bmp;
extern crate byteorder;

use byteorder::{ByteOrder, BigEndian, WriteBytesExt};
use bmp::{Image, Pixel};
use nom::{ToUsize, IResult};
use nom::number::streaming::{be_u8, be_u32};
use nom::bytes::streaming::*;
use nom::multi::count;
use anyhow::{Context, Result};
use tribool::Tribool;
use rgb::FromSlice;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::mem;

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

fn parse(buffer: &[u8]) -> Result<Spi> {
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

/*
pub fn convert_spi0(spi: &Spi, index: u32) -> Result<()> {
    let mut out = File::create(format!("C:\\Users\\yuno\\Documents\\josette\\spi_{:02x}.spi", index))?;
    spi.write(&mut out)?;

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

    assert!(spi.header.magic == "SPI0");

    // printall(s1);
    // printall(s2);
    // printall(s3);

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
        // println!("test! {} {} {}", s3[s3_offset] & 0x80, shift, s3[s3_offset] & 0x80 >> shift);
        if (s3[s3_offset] & 0x80 >> shift) != 0 {
            Tribool::True
        }
        else {
            Tribool::False
        }
    };

    while output.len() < spi.header.u1 {
        // println!("header {} target {} output {} data {} {} {} {}", spi.header.u1, spi.header.u1, output.len(), spi.data.len(), s1.len(), s2.len(), s3.len());
        let found = test_found();
        if found.is_indeterminate() {
            // println!("ranout");
            break;
        }

        if found.is_true() {
            let byte = s2[s2_offset];
            s2_offset += 1;
            output.push(byte);
        }
        else {
            let mut a = (s2[s2_offset] as u32) >> 4;
            let b = s2[s2_offset] as usize;
            let c = s2[s2_offset + 1] as usize;
            // println!("ff a b c {:02x} {:02x} {:02x}", a, b, c);
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
            let mut pos = output.len() - c - (b & 0xF) * 0x100 - 1;
            a += 3;
            // println!("finala {:02x}", a);
            while a > 0 {
                a -= 1;
                // run-length encoding
                // TODO sized buffer
                // println!("len {} b {:02x} c {:02x} write {:02x}", output.len(), b, c, output[pos]);
                output.push(output[pos]);
                pos += 1;
            }
        }
    }

    // println!("done");

    // write_files(&output, &buffer, pos, 32, 32)
    Ok(())
}
*/

pub fn decompress_spi1(spi: &Spi) -> Result<Vec<u8>> {
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

    // printall(s1);
    // printall(s2);
    // printall(s3);

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
        // println!("test! {} {} {}", s3[s3_offset] & 0x80, shift, s3[s3_offset] & 0x80 >> shift);
        if (s3[s3_offset] & 0x80 >> shift) != 0 {
            Tribool::True
        }
        else {
            Tribool::False
        }
    };

    while output.len() < spi.header.u1 {
        // println!("header {} target {} output {} data {} {} {} {}", spi.header.u1, spi.header.u1, output.len(), spi.data.len(), s1.len(), s2.len(), s3.len());
        let found = test_found();
        if found.is_indeterminate() {
            // println!("ranout");
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
                    // println!("tt s2[{:02x}]={:02x} col={:02x}", s2_offset, it, color_index);
                    it
                },
                Tribool::False => {
                    // println!("tf s1[{:02x}]={:02x}", s1_offset, s1[s1_offset]);
                    let color_index = if !is_other {
                        is_other = true;
                        (s1[s1_offset] >> 4) as usize
                    } else {
                        let thing = s1[s1_offset];
                        s1_offset += 1;
                        is_other = false;
                        (thing & 0xF) as usize
                    };

                    // println!("tf col={:02x}", color_index);
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
            // println!("ff a b c {:02x} {:02x} {:02x}", a, b, c);
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
            let mut pos = output.len() - c - (b & 0xF) * 0x100 - 1;
            a += 3;
            // println!("finala {:02x}", a);
            while a > 0 {
                a -= 1;
                // run-length encoding
                // TODO sized buffer
                // println!("len {} b {:02x} c {:02x} write {:02x}", output.len(), b, c, output[pos]);
                output.push(output[pos]);
                pos += 1;
            }
        }
    }

    Ok(output)
}

fn write_bmp(decompressed: &[u8], spi: &Spi, palette: &Palette, index: u32) -> Result<()> {
    // let mut out = File::create(format!("C:\\Users\\yuno\\Documents\\josette\\output_{:02x}.bin", pos))?;
    // out.write_all(&decompressed).unwrap();

    // let mut palimg = Image::new(16, 16);
    // for (col, (i, (x, y))) in palette2.iter().zip(palimg.coordinates().enumerate()) {
    //     palimg.set_pixel(x, y, px!(col.r, col.g, col.b));
    // }
    // palimg.save(format!("C:\\Users\\yuno\\Documents\\josette\\palette_{:02x}.bmp", pal))?;

    let mut total_width = 0;
    let mut total_height = 0;

    let mut cur = decompressed;
    let mut i = 0;
    while cur.len() > 2 {
        let offset_x = BigEndian::read_u16(&cur[0..2]);
        let offset_y = BigEndian::read_u16(&cur[2..4]);
        let width = BigEndian::read_u16(&cur[4..6]);
        let height = BigEndian::read_u16(&cur[6..8]);

        // TODO backgrounds?
        if offset_x > 256 {
            println!("Skip {} {}", width, height);
            return Ok(());
        }

        let offset = (width * height) as usize;
        total_width = std::cmp::max(total_width, offset_x + width);
        total_height = std::cmp::max(total_height, offset_y + height);
        cur = &cur[8+offset..];

        i += 1;
    }

    let mut img = Image::new(total_width as u32, total_height as u32);

    let mut cur = decompressed;
    i = 0;
    while cur.len() > 2 {
        let offset_x = BigEndian::read_u16(&cur[0..2]) as u32;
        let offset_y = BigEndian::read_u16(&cur[2..4]) as u32;
        let width = BigEndian::read_u16(&cur[4..6]) as u32;
        let height = BigEndian::read_u16(&cur[6..8]) as u32;
        let offset = (width * height) as usize;
        let bitmap_part = &cur[8..8+offset];

        for (i, by) in bitmap_part.iter().enumerate() {
            let x = (i as u32 % width);
            let y = (i as u32 / width);
            let col = palette.colors[*by as usize];
            img.set_pixel(offset_x + x, offset_y + y, px!(col.r, col.g, col.b));
        }

        cur = &cur[8+offset..];
        i += 1;
    }

    img.save(format!("C:\\Users\\yuno\\Documents\\josette\\spi_{}.bmp", index))?;

    Ok(())
}

pub struct ObjInfo {
    pub offset1: u16,
    pub offset2: u16,
    pub u1: u32,
    pub u2: u32,
    pub u3: u32,
}

pub struct ObjDef {
    pub frames_offset: u32,
    pub u1: u16,
    pub u2: u16,
    pub u3: u16,
    pub u4: u16,
    pub u5: u32,
    pub frame_count: u8,
    pub pad1: u8,
    pub pad2: u8,
    pub pad3: u8,
    pub frames: Vec<Frame>
}

pub struct Frame {
    pub idx: u16,
    pub kind: u8,
    pub id: u8,
    pub delay: u8,
    pub u2: u8,
    pub x: i16,
    pub y: i16,
    pub u5: u16,
    pub u6: u8,
    pub u7: u8,
}

pub(crate) fn get_slice<'a>(slice: &'a [u8], offset: u32, size: u32) -> &'a [u8] {
    let offset = offset as usize;
    let size = size as usize;

    &slice[offset..(offset+size)]
}

pub(crate) unsafe fn transmute_slice<'a, T>(slice: &'a [u8], offset: u32, size: u32) -> &'a [T] {
    let t_slice = get_slice(slice, offset, size);
    std::slice::from_raw_parts(t_slice.as_ptr() as *const _, size as usize / mem::size_of::<T>())
}

pub struct Palette {
    colors: Vec<rgb::RGBA16>
}

fn parse_objinfos(buffer: &[u8]) -> Result<()>{
    // headers
    let offset = 0x000f27e0;

    // frame count header offset
    let defs_offset = 0x000fd180;

    // frame data offset
    let frames_base_offset = 0x00105220;

    // spi offsets offset
    let spi_offset_offset = 0x00133ac0;

    // spi data offset
    let spi_base_offset = 0x0013d2e0;

    let mut objinfos = Vec::new();
    let mut defs = Vec::new();
    let mut spis = Vec::new();
    let mut palettes = Vec::new();

    for i in 0..0x9B4 {
        let ind = offset + i * 0x10;
        let offset1 = BigEndian::read_u16(&buffer[ind..ind+2]);
        let offset2 = BigEndian::read_u16(&buffer[ind+2..ind+4]);
        let u1 = BigEndian::read_u32(&buffer[ind+4..ind+8]);
        let u2 = BigEndian::read_u32(&buffer[ind+8..ind+12]);
        let u3 = BigEndian::read_u32(&buffer[ind+12..ind+16]);
        objinfos.push(ObjInfo { offset1, offset2, u1, u2, u3 });
    }

    for i in 0..1645 {
        let ind = defs_offset + i * 0x14;
        let frames_offset = BigEndian::read_u32(&buffer[ind..ind+4]);
        let u1 = BigEndian::read_u16(&buffer[ind+4..ind+6]);
        let u2 = BigEndian::read_u16(&buffer[ind+6..ind+8]);
        let u3 = BigEndian::read_u16(&buffer[ind+8..ind+10]);
        let u4 = BigEndian::read_u16(&buffer[ind+10..ind+12]);
        let u5 = BigEndian::read_u32(&buffer[ind+12..ind+16]);
        let frame_count = buffer[ind+16];
        println!("frames offset {:02x} {} ind {:02x}", frames_offset, i, ind);

        let mut frames = Vec::new();
        for j in 0..frame_count {
            let ind = frames_base_offset + (frames_offset as usize) + (j as usize) * 0xe;
            let idx = BigEndian::read_u16(&buffer[ind+0..ind+2]);
            let kind = buffer[ind+2];
            let id = buffer[ind+3];
            let delay = buffer[ind+4];
            let u2 = buffer[ind+5];
            let x = BigEndian::read_i16(&buffer[ind+6..ind+8]);
            let y = BigEndian::read_i16(&buffer[ind+8..ind+10]);
            let u5 = 0;
            let u6 = 0;
            let u7 = 0;
            frames.push(Frame { idx, kind, id, delay, u2, x, y, u5, u6, u7 });
        }

        defs.push(ObjDef { frames_offset, u1, u2, u3, u4, u5, frame_count, pad1: 0, pad2: 0, pad3: 0, frames: frames });
    }

    for i in 0..0xf81 {
        let mut off = 0;
        let mut spi_offset = 0;
        loop {
            let ind = spi_offset_offset + (i + off) * 8;
            spi_offset = BigEndian::read_u32(&buffer[ind..ind+8]);
            if spi_offset & 1 == 0 {
                break
            }
            off += 1;
        }

        println!("spi offset {:02x}: {:02x} {:02x}", i, spi_offset, spi_base_offset + spi_offset as usize);

        let (_, spi) = spi(&buffer[spi_base_offset+spi_offset as usize..]).map_err(|e| anyhow!("Parsing failed! {:?}", e))?;
        spis.push(spi);
    }

    for i in 0..0x60 {
        let pal_begin = 0xf27e0 + (((i * 0x200) + 0x8078D1C0) - 0x80400000);
        let palette = &buffer[pal_begin..pal_begin+0x200];

        let mut colors = Vec::new();
        for i in 0..0x100 {
            let ind = i * 2;

            // RGB5551
            let by = BigEndian::read_u16(&palette[ind..ind+2]);
            let r = (((by >> 11) & 0x1F) * 255 + 15) / 31;
            let g = (((by >> 6) & 0x1F) * 255 + 15) / 31;
            let b = (((by >> 1) & 0x1F) * 255 + 15) / 31;
            let a = (by & 0x0001) * 255;

            colors.push(rgb::RGBA::new(r, g, b, a));
        }

        palettes.push(Palette { colors: colors })
    }

    for (i, spi) in spis.iter().enumerate() {
        println!("spi {}: {} {:04x}", i, spi.header.magic, spi.header.u1);
        if spi.header.magic == "SPI1" {
            let decomp = decompress_spi1(spi)?;
            write_bmp(&decomp, spi, &palettes[3], i as u32);
        }
    }

    for (i, obj) in objinfos.iter().enumerate() {
        println!("def {}: {:04x} {:04x} {:08x} {:08x} {:08x}", i, obj.offset1, obj.offset2, obj.u1, obj.u2, obj.u3);
    }

    for (i, def) in defs.iter().enumerate() {
        println!("OBJ {}: {:08x}, {}", i, def.frames_offset, def.frame_count);

        for frame in def.frames.iter() {
            println!("\t{:08} {:08x} {:08x} {} {} {}", frame.idx, frame.kind, frame.id, frame.x, frame.y, frame.delay);
        }
    }

    Ok(())
}

fn main() {
    let mut f = File::open("C:\\Users\\yuno\\Documents\\Wonder Project J2 - Koruro no Mori no Jozet (Japan).z64").context("Unable to open file").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).context("Unable to read file").unwrap();

    parse_objinfos(&buffer);

    // let spi0 = "SPI0";
    // for (pos, _) in buffer.windows(4).enumerate().filter(|(_, window)| window == &spi0.as_bytes())
    // {
    //     // println!("SPI0! {:02x}", pos);
    //     convert_spi0(&buffer, pos).unwrap();
    // }

    // let spi1 = "SPI1";
    // for (pos, _) in buffer.windows(4).enumerate().filter(|(_, window)| window == &spi1.as_bytes())
    // {
    //     println!("SPI1! {:02x}", pos);
    //     convert_spi1(&buffer, pos).unwrap();
    // }
}
