#[macro_use] extern crate anyhow;
extern crate nom;
extern crate hexyl;
extern crate tribool;
extern crate rgb;
#[macro_use] extern crate bmp;
extern crate byteorder;
#[macro_use] extern crate bitflags;
extern crate clap;

mod convert;
mod obj;
mod spi;
mod writeable;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use writeable::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Path to Wonder Project J2 ROM (.z64)
    rompath: PathBuf,

    /// Output path
    outpath: PathBuf,

    /// Palette to use when exporting
    #[clap(short, long, default_value_t = 0)]
    palette: usize,
}

fn main() {
    let args = Args::parse();

    let mut f = File::open(&args.rompath).context("Unable to open file").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).context("Unable to read file").unwrap();

    obj::parse_objinfos(&args, &buffer);

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
