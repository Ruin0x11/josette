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
use std::path::Path;
use writeable::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long)]
    name: String,

    /// Number of times to greet
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let mut f = File::open("C:\\Users\\yuno\\Documents\\Wonder Project J2 - Koruro no Mori no Jozet (Japan).z64").context("Unable to open file").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).context("Unable to read file").unwrap();

    obj::parse_objinfos(&buffer);

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
