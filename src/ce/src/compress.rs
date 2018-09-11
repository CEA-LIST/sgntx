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


use std::io;
use std::fs;
use std::fmt;
use std::num;
use std::io::BufRead;
use std::io::Write;
use std::str::FromStr;
use std::path;

use rand;

use shared;

extern "C" {
    fn aes_gcm_encrypt(key:  *const u8,
                       from: *const u8,
                       size: u64,
                       to:   *mut u8,
                       iv:   *mut u8,
                       mac:  *mut u8);
}


fn write_block<T: Write, R: rand::Rng>( writer:  &mut io::BufWriter<T>,
                                        content: &[shared::Key],
                                        buffer:  &mut [u8],
                                        rng:     &mut R ) -> io::Result<usize> {
    // convert vector to &[u8].
    let slice = shared::as_u8_slice( content );
    
    // Encrypt the block.
    let blk_size = {
        // let aad = [0u8;0];
        let (mut hdr,buf) = buffer.split_at_mut( 32 );
        let size  = slice.len();
    
        // Initialize block header.
        let mut iv = [0u32;3];
        for i in iv.iter_mut() {
            *i = rng.gen::<u32>();
        }
        let header = shared::Header::new( size as u32, iv );

        hdr.copy_from_slice( header.as_slice() );

         unsafe {
             aes_gcm_encrypt( shared::AES_KEY.as_ptr(),
                              slice.as_ptr(),
                              slice.len() as u64,
                              buf.as_mut_ptr(),
                              hdr[4..16].as_mut_ptr(),
                              hdr[16..].as_mut_ptr() );
        } 
        header.blk_size() as usize
    };
    
    // write the block.
    writer.write( &buffer[..blk_size] )
}





pub fn compress( inp_path: &path::PathBuf, out_path: &path::PathBuf, keys_per_blk: u32 ) -> Result<(),Error> {
    let from   = try!( fs::File::open(&inp_path) );
    let to     = try!( fs::OpenOptions::new()
                       .create(true)
                       .truncate(true)
                       .write(true)
                       .open(&out_path) );
    
    // let mut reader = io::BufReader::with_capacity( 8*1024, from );
    let mut reader = io::BufReader::new( from );
    let mut writer = io::BufWriter::new( to );

    let mut content = Vec::with_capacity( keys_per_blk as usize);
    
    let mut buffer = {
        let size = shared::block_size( keys_per_blk ) as usize;
        let mut b = Vec::with_capacity( size );
        unsafe { b.set_len( size ) };
        b.into_boxed_slice()
    };
                 
    // Initialize random number generator.
    let mut rng = rand::thread_rng();
 
    let mut line = String::with_capacity( 64 );
    
    loop {
        let len = try!( reader.read_line( &mut line ) );
        if len == 0 {
            if content.len() > 0 {
                try!( write_block( &mut writer, &content, &mut buffer, &mut rng ) );
            }
            break
        } else {
            let line = line.trim();
            if !line.starts_with('#') && !line.is_empty() {
                let data = try!( parse_line( line ) );
                content.push( data );
                if content.len() == content.capacity() {
                    try!( write_block( &mut writer, &content, &mut buffer, &mut rng ) );
                    content.clear();
               }
            }
        }
        line.clear();
    }
    
    Ok(())
}



#[derive(Debug)]
pub enum Error {
    InvalidLine,
    InvalidFormat,
    Shared(shared::Error),
    Int(num::ParseIntError),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Int(ref err)    => write!(f, "{}", err),
            Error::Shared(ref err) => write!(f, "{}", err),
            Error::Io(ref err)     => write!(f, "{}", err),
            Error::InvalidFormat   => write!(f, "Invalid VCF Format"),
            Error::InvalidLine     => write!(f, "Invalid Line Format"),            
        }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::Int(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<shared::Error> for Error {
    fn from(err: shared::Error) -> Error {
        Error::Shared(err)
    }
}


pub fn parse_line(line: &str) -> Result<shared::Key,Error> {
    // Retreive the necessary fields.
    let mut iter = line.split('\t');
    // Read CHROM
    let chrom: u8 =
        match iter.next() {
            Some(s) => try!( s.parse() ),
            None    => return Err(Error::InvalidLine),
        };
    // Read POS
    let pos: u32 =
        match iter.next() {
            Some(s) => try!( s.parse() ),
            None    => return Err(Error::InvalidLine),
        };
   // Read ID
    let id: u64 =
        match iter.next() {
            Some(".") => 0 ,
            Some(s) => {
                if s.len() > 2 && &s[..2] == "rs" { try!( s[2..].parse() ) }
                else { return Err(Error::InvalidFormat) }
            },
            None    => return Err(Error::InvalidLine),
    };
    // Read REF.
    let refe =
        match iter.next() {
            Some(s) => try!( shared::Base::from_str(s) ),
            None    => return Err(Error::InvalidLine),
        };
    // Read ALT.
    let alt =
        match iter.next() {
            Some(s) => try!( shared::Base::from_str(s) ),
            None    => return Err(Error::InvalidLine),
        };
    // Skip QUAL and FILTER and read TYP
    let typ = 
        match iter.nth(2) {
            Some(s) => try!( shared::Typ::from_str( s ) ),
            _       => return Err(Error::InvalidLine),
        };

    Ok( shared::Key::new(chrom, pos, id, refe, alt, typ) )
}
