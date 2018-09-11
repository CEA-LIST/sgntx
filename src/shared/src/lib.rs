//
//   (C) Copyright 2017 CEA LIST. All Rights Reserved.
//   Contributor(s): Thibaud Tortech & Sergiu Carpov
//
//   This software is governed by the CeCILL-C license under French law and
//   abiding by the rules of distribution of free software.  You can  use,
//   modify and/ or redistribute the software under the terms of the CeCILL-C
//   license as circulated by CEA, CNRS and INRIA at the following URL
//   "http://www.cecill.info".
//
//   As a counterpart to the access to the source code and  rights to copy,
//   modify and redistribute granted by the license, users are provided only
//   with a limited warranty  and the software's author,  the holder of the
//   economic rights,  and the successive licensors  have only  limited
//   liability.
//
//   The fact that you are presently reading this means that you have had
//   knowledge of the CeCILL-C license and that you accept its terms.
//


#![no_std]


use core::fmt;
use core::slice;
use core::mem;
use core::u64;
use core::u32;
use core::u16;

use core::cmp::Ordering;
use core::str::FromStr;


pub const KEYS_PER_BLOCK_DEFAULT: u32 = 2080;
pub const ITER_FACTOR_DEFAULT: u32 = 4;

pub const AES_KEY: [u8; 16] =
    [0x4c, 0x86, 0xaa, 0xf6, 0xaf, 0xc9, 0x5e, 0x87, 0xa6, 0x85, 0x18, 0xdf, 0x8a, 0xe7, 0x58, 0x29];


#[derive(Clone,Copy,Debug)]
#[repr(u8)]
pub enum Kind {
    Control = 0,
    Case    = 1,
}


#[derive(Debug)]
pub enum Error {
    InvalidBase,
    InvalidType,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Error::InvalidBase => "Invalid Base",
            Error::InvalidType => "Invalid Type",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone,Copy,Debug)]
#[repr(u8)]
pub enum Base { A = 0, C, G, T, N }

impl fmt::Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Base::A => "A",
            Base::C => "C",
            Base::G => "G",
            Base::T => "T",
            Base::N => "N",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Base {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Base::A),
            "C" => Ok(Base::C),
            "G" => Ok(Base::G),
            "T" => Ok(Base::T),
            "N" => Ok(Base::N),
            _ =>   Err(Error::InvalidBase),
        }
    }
}


#[derive(Copy,Clone)]
#[repr(u8)]
pub enum Typ { Heterozygous = 0, Homozygous }

impl fmt::Display for Typ {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Typ::Heterozygous => "heterozygous",
            Typ::Homozygous   => "homozygous",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Typ {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "heterozygous" => Ok(Typ::Heterozygous),
            "homozygous"   => Ok(Typ::Homozygous),
            _              => Err(Error::InvalidType),
        }
    }
}

#[derive(Clone,Copy,Debug)]
pub struct Key(u16, u16, u16, u16, u16);

pub const KEY_MIN: Key = Key( u16::MIN, u16::MIN, u16::MIN, u16::MIN, u16::MIN );
pub const KEY_MAX: Key = Key( u16::MAX, u16::MAX, u16::MAX, u16::MAX, u16::MAX );

impl Key {

    //encode (refe, alt) pairs on 5 bits
    fn encode_base_pair(refe: Base, alt: Base) -> u8 {
        let refe_i = refe as u8;
        let alt_i = alt as u8;

        (refe_i * 4 + alt_i - (alt_i > refe_i) as u8)
    }    

    fn decode_base_pair(refe_alt: u8) -> (Base, Base) {
        let refe = unsafe{ mem::transmute( (refe_alt / 4) as u8 ) };
        let alt = unsafe {
            let t = refe_alt % 4;
            mem::transmute(t as u8 + (t >= refe_alt / 4) as u8)
        };
        (refe, alt)
    }


    pub fn new(chrom: u8, pos: u32, id: u64, refe: Base, alt: Base, typ: Typ ) -> Key {
        let a = (pos >> 16) as u16;
        let b = pos as u16;

        let refe_alt = Key::encode_base_pair(refe, alt);
        let id_top = (id >> 32) as u16;
        let c = (chrom as u16 & 0b11111) | (refe_alt as u16 & 0b11111) << 5 | (typ as u16) << 10 | id_top << 11;

        let d = (id >> 16) as u16;
        let e = id as u16;

        Key(a,b,c,d,e)
    }

    pub fn chrom(&self) -> u8 {
        (self.2 & 0b11111) as u8
    }

    pub fn pos(&self) -> u32 {
        ((self.0 as u32) << 16) + self.1 as u32
    }

    pub fn id(&self) -> u32 {
        ((self.2 >> 11) & 0b11111) as u32 + ((self.3 as u32) << 16) + self.4 as u32
    }

    pub fn refe_alt(&self) -> (Base, Base) {
        Key::decode_base_pair(((self.2 >> 5) & 0b11111) as u8)
    }

    pub fn typ(&self) -> Typ {
        unsafe { mem::transmute( ((self.2 >>10) & 1) as u8 ) }
    }

    pub fn chrom_refe_alt(&self) -> u16 {
        (self.2 & 0b11111_11111)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id = self.id();
        let (refe, alt )= self.refe_alt();
        if id == 0 {
            write!(f, "{:2}\t{:10}\t.\t{}\t{}", self.chrom(), self.pos(), refe, alt )
        } else {
            write!(f, "{:2}\t{:10}\trs{}\t{}\t{}", self.chrom(), self.pos(), id, refe, alt )
        }
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Key) -> bool {
        (self.pos() == other.pos()) & (self.chrom_refe_alt() == other.chrom_refe_alt())
    }
}

impl Eq for Key {}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Key) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Key) -> Ordering {
        self.pos().cmp(&other.pos()).then(self.chrom_refe_alt().cmp(&other.chrom_refe_alt()))
    }
}

pub trait Hash {
    fn hash(&self) -> u64;
}

const GOLDEN_RATIO_64:       u64 = 0x61c8864680b583eb;

impl Hash for Key {
    fn hash(&self) -> u64 {
        (self.pos() as u64 | ((self.chrom_refe_alt() as u64) << 32)) * GOLDEN_RATIO_64
    }
}



// Compressed file block header.
pub struct Header {
    size: u32,
    iv:   [u32;3],
    mac:  [u8;16],
}

pub fn block_size( nb_keys: u32 ) -> usize {
    size( nb_keys ) + mem::size_of::<Header>()
}

pub fn size( nb_keys: u32 ) -> usize {
    nb_keys as usize * mem::size_of::<Key>()
}


impl Header {
    pub fn new( size: u32, iv: [u32;3]) -> Header {
        Header { size: size, iv: iv, mac: [0u8;16] }
    }

    pub fn as_slice(&self) -> &[u8] {
        as_u8_slice( self )
    }

    pub fn blk_size(&self) -> u32 {
        self.size + mem::size_of::<Header>() as u32
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn iv(&self) -> &[u8] {
        as_u8_slice( &self.iv )
    }
    
    pub fn mac(&self) -> &[u8;16] {
        &self.mac
    }
    
    pub fn nb_keys(&self) -> u32 {
        assert_eq!( self.size % mem::size_of::<Key>() as u32, 0);
        self.size / mem::size_of::<Key>() as u32
    }
}


pub fn as_u8_slice<T:?Sized>(p: &T) -> &[u8] {
    unsafe {
        slice::from_raw_parts( p as *const T as *const u8,
                               mem::size_of_val::<T>(p) )
    }
}


pub fn as_u8_slice_mut<T:?Sized>(p: &mut T) -> &mut [u8] {
    unsafe {
        slice::from_raw_parts_mut( p as *mut T as *mut u8,
                                   mem::size_of_val::<T>(p) )
    }
}


