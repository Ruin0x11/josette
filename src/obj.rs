use nom::{ToUsize, IResult};
use nom::number::streaming::{be_u8, be_u32};
use nom::bytes::streaming::*;
use nom::multi::count;
use anyhow::{Context, Result};
use byteorder::{ByteOrder, BigEndian, WriteBytesExt};
use crate::Args;
use image::{Rgba, RgbaImage};

bitflags! {
    #[repr(C)]
    pub struct ObjInfoFlags: u16 {
        const EMPTY    = 0b0000000000000000;
        const HASEXTRA = 0b0000000000000001;
        const LOOP     = 0b0000000000000010;
        const UNK3     = 0b0000000000000100;
        const UNK4     = 0b0000000000001000;
        const BG2FG    = 0b0000000000010000;
        const UNK6     = 0b0000000000100000;
        const UNK8     = 0b0000000001000000;
        const FG2BG    = 0b0000000010000000;
        const UNK10    = 0b0000000100000000;
        const UNK12    = 0b0000001000000000;
        const UNK13    = 0b0000010000000000;
        const UNK14    = 0b0000100000000000;
        const UNK15    = 0b0001000000000000;
        const UNK16    = 0b0010000000000000;
        const UNK17    = 0b0100000000000000;
        const UNK18    = 0b1000000000000000;
    }
}

pub struct ObjInfo {
    pub offset1: u16,
    pub offset2: u16,
    pub u1: u32,
    pub flags: ObjInfoFlags,
    pub u2: u16,
    pub u3: u16,
    pub obj_count: u8,
    pub extra_obj_count: u8,
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
    pub spi_idx: u16,
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

fn get_slice<'a>(slice: &'a [u8], offset: u32, size: u32) -> &'a [u8] {
    let offset = offset as usize;
    let size = size as usize;

    &slice[offset..(offset+size)]
}

unsafe fn transmute_slice<'a, T>(slice: &'a [u8], offset: u32, size: u32) -> &'a [T] {
    let t_slice = get_slice(slice, offset, size);
    std::slice::from_raw_parts(t_slice.as_ptr() as *const _, size as usize / std::mem::size_of::<T>())
}

pub struct Palette {
    pub index: usize,
    pub colors: Vec<Rgba<u8>>
}

pub fn parse_objinfos(args: &Args, buffer: &[u8]) -> Result<()>{
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
        let flags = BigEndian::read_u16(&buffer[ind+8..ind+10]);
        let u2 = BigEndian::read_u16(&buffer[ind+10..ind+12]);
        let u3 = BigEndian::read_u16(&buffer[ind+12..ind+14]);
        let obj_count = buffer[ind+14];
        let extra_obj_count = buffer[ind+15];
        objinfos.push(ObjInfo { offset1, offset2, u1, flags: ObjInfoFlags { bits: flags }, u2, u3, obj_count, extra_obj_count });
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

        if args.debug {
            println!("frames offset {:02x} {} ind {:02x}", frames_offset, i, ind);
        }

        let mut frames = Vec::new();
        for j in 0..frame_count {
            let ind = frames_base_offset + (frames_offset as usize) + (j as usize) * 0xe;
            let spi_idx = BigEndian::read_u16(&buffer[ind+0..ind+2]);
            let kind = buffer[ind+2];
            let id = buffer[ind+3];
            let delay = buffer[ind+4];
            let u2 = buffer[ind+5];
            let x = BigEndian::read_i16(&buffer[ind+6..ind+8]);
            let y = BigEndian::read_i16(&buffer[ind+8..ind+10]);
            let u5 = BigEndian::read_u16(&buffer[ind+10..ind+12]);
            let u6 = buffer[12];
            let u7 = buffer[13];
            frames.push(Frame { spi_idx, kind, id, delay, u2, x, y, u5, u6, u7 });
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

        if args.debug {
            println!("spi offset {:02x}: {:02x} {:02x}", i, spi_offset, spi_base_offset + spi_offset as usize);
        }

        let (_, spi) = crate::spi::spi(&buffer[spi_base_offset+spi_offset as usize..]).map_err(|e| anyhow!("Parsing failed! {:?}", e))?;
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

            colors.push(Rgba::<u8>([r as u8, g as u8, b as u8, a as u8]));
        }

        palettes.push(Palette { index: i, colors: colors })
    }

    let palette = &palettes[args.palette];

    for (i, pal) in palettes.iter().enumerate() {
        let mut palimg = RgbaImage::new(256, 1);
        for (x, col) in pal.colors.iter().enumerate() {
            palimg.put_pixel(x as u32, 0, *col);
        }
        palimg.save(args.outpath.join(format!("palette/palette_{:02}.png", i)))?;
    }

    for (i, spi) in spis.iter().enumerate() {
        if args.debug {
            println!("spi {}: {} {:04x}", i, spi.header.magic, spi.header.u1);
        }

        if spi.header.magic == "SPI1" {
            let decomp = crate::convert::decompress_spi1(spi)?;
            crate::convert::write_spi1_png(&args, &decomp, spi, palette, i as u32);
        }
    }

    for (i, obj) in objinfos.iter().enumerate() {
        if args.debug {
            println!("def {}: {:04x} {:04x} {:08x} objs={} extra={} flags={:?}", i, obj.offset1, obj.offset2, obj.u1, obj.obj_count, obj.extra_obj_count, obj.flags);
        }
    }

    for (i, def) in defs.iter().enumerate() {
        if args.debug {
            println!("OBJ {}: {:08x}, {}", i, def.frames_offset, def.frame_count);

            for frame in def.frames.iter() {
                println!("\t{:0>8} {:08x} {:08x} {} {} {} {} {} {} {}", frame.spi_idx, frame.kind, frame.id, frame.x, frame.y, frame.delay, frame.u2, frame.u5, frame.u6, frame.u7);
            }
        }

        crate::convert::write_anim_png(&args, &def, i, &spis, &palette)?;
    }

    Ok(())
}
