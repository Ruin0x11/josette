use image::{ImageBuffer, Rgba, RgbaImage, Pixel};
use rgb::FromSlice;
use anyhow::{Context, Result};
use byteorder::{ByteOrder, BigEndian, WriteBytesExt};
use tribool::Tribool;
use std::mem;
use std::collections::HashMap;
use crate::Args;
use crate::spi::Spi;
use crate::obj::{Palette, ObjDef};
use crate::writeable::Writeable;

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

fn get_spi_size(spi: &Spi, decompressed: &[u8]) -> (u32, u32) {
    let mut total_width = 0;
    let mut total_height = 0;

    let mut cur = decompressed;
    let mut i = 0;
    while cur.len() > 2 {
        let offset_x = BigEndian::read_u16(&cur[0..2]);
        let offset_y = BigEndian::read_u16(&cur[2..4]);
        let width = BigEndian::read_u16(&cur[4..6]);
        let height = BigEndian::read_u16(&cur[6..8]);

        let offset = (width * height) as usize;
        total_width = std::cmp::max(total_width, offset_x + width);
        total_height = std::cmp::max(total_height, offset_y + height);
        cur = &cur[8+offset..];

        i += 1;
    }

    (total_width as u32, total_height as u32)
}

fn write_spi_partial(img: &mut RgbaImage, spi: &Spi, decompressed: &[u8], palette: &Palette, px: u32, py: u32) {
    let mut cur = decompressed;
    let mut i = 0;
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
            img.put_pixel(offset_x + x + px, offset_y + y + py, col);
        }

        cur = &cur[8+offset..];
        i += 1;
    }
}

pub fn write_spi1_png(args: &Args, decompressed: &[u8], spi: &Spi, palette: &Palette, index: u32) -> Result<()> {
    // TODO backgrounds?
    let offset_x = BigEndian::read_u16(&decompressed[0..2]);
    if offset_x > 256 {
        println!("Skip {}", offset_x);
        return Ok(());
    }

    let (total_width, total_height) = get_spi_size(spi, decompressed);
    let mut img = RgbaImage::new(total_width as u32, total_height as u32);

    write_spi_partial(&mut img, &spi, decompressed, &palette, 0, 0);
    img.save(args.outpath.join(format!("spi1/spi1_pal{:0>2}_{:0>8}.png", palette.index, index)))?;
    Ok(())
}

pub fn write_anim_png(args: &Args, def: &ObjDef, index: usize, spis: &[Spi], palette: &Palette) -> Result<()> {
    let mut total_width = 0;
    let mut total_height = 0;
    let mut spi_data = HashMap::new();
    let mut ey = 0;

    for frame in def.frames.iter() {
        if (frame.spi_idx & 0x8000) != 0 {
            continue;
        }

        let spi = &spis[frame.spi_idx as usize];
        let decomp = crate::convert::decompress_spi1(spi)?;

        let (w, h) = get_spi_size(spi, &decomp);
        total_width += w + frame.x.abs() as u32;
        total_height = std::cmp::max(total_height, h + frame.y as u32);
        ey = std::cmp::max(ey, frame.y);

        spi_data.insert(frame.spi_idx, decomp);
    }

    let mut img = RgbaImage::new(total_width, total_height + 100);
    let mut x = 0;

    for frame in def.frames.iter() {
        if (frame.spi_idx & 0x8000) != 0 {
            continue;
        }

        let spi = &spis[frame.spi_idx as usize];
        let decomp = &spi_data[&frame.spi_idx];
        let (w, h) = get_spi_size(spi, &decomp);

        let px = x;
        // println!("th={} y={} h={}", total_height, frame.y, h);
        let py = ey as i32 - (frame.y as i32);
        // println!("{},{}", px, py);

        write_spi_partial(&mut img, spi, &decomp, &palette, px as u32, py as u32);
        x += w as i32;
    }

    img.save(args.outpath.join(format!("anim/anim_pal{:0>2}_{:0>8}.png", palette.index, index)))?;
    Ok(())
}
