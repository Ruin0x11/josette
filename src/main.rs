#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use] extern crate anyhow;
extern crate nom;
extern crate hexyl;
extern crate tribool;
extern crate rgb;
extern crate image;
extern crate byteorder;
#[macro_use] extern crate bitflags;
extern crate clap;

mod convert;
mod obj;
mod spi;
mod writeable;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs::{self, File};
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

    /// Print debugging information
    #[clap(short, long)]
    debug: bool,

    /// Don't output SPI1 data
    #[clap(short, long)]
    no_spi1: bool,
}

fn main() {
    let args = Args::parse();

    let mut f = File::open(&args.rompath).context("Unable to open file").unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).context("Unable to read file").unwrap();

    fs::create_dir_all(args.outpath.join("palette")).unwrap();
    fs::create_dir_all(args.outpath.join("spi1")).unwrap();
    fs::create_dir_all(args.outpath.join("anim")).unwrap();

    obj::parse_objinfos(&args, &buffer);
}
