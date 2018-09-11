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
#![feature(alloc)]
#![feature(core_intrinsics)]
#![feature(integer_atomics)]
#![feature(lang_items)]
#![feature(global_allocator)]


extern crate sgx_types;
extern crate sgx_tcrypto;
extern crate sgx_trts;
extern crate sgx_alloc;
use sgx_tcrypto::*;



extern crate alloc;

extern crate shared;


use core::mem;
use core::slice;
use core::fmt;

#[macro_use]
mod console;
mod ocall;
mod types;
mod chisquare;


mod spin;
mod hashmap;
mod imp_hashmap;
use imp_hashmap as imp;


#[global_allocator]
static A: sgx_alloc::System = sgx_alloc::System;



static mut DATA: Option<imp::GlobalData> = None;


fn data() -> &'static mut imp::GlobalData {
    unsafe { DATA.as_mut().unwrap() }
}


#[lang="panic_fmt"]
#[no_mangle]
pub extern fn enclave_panic(msg: fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("{}:{}: {}", file, line, msg);
    sgx_trts::trts::rsgx_abort();
}



#[no_mangle]
pub extern "C" fn encl_init(nb_control:     u32,
                            nb_case:        u32,
                            keys_per_block: u32,
                            nb_keys:        u32,
                            snp_cnt:        usize,
                            output_allele_freq: bool ) {
    // Init
    unsafe {
        DATA = Some( imp::GlobalData::new( nb_control as usize,
                                           nb_case as usize,
                                           keys_per_block,
                                           nb_keys,
                                           snp_cnt,
                                           output_allele_freq ) );
    }
}

#[no_mangle]
pub extern "C" fn encl_register(fid:  u32, kind: shared::Kind ) {
    data().register( fid as usize, kind )
}



#[no_mangle]
pub extern "C" fn encl_begin(fid: u32) -> u32 {
    data().begin( fid as usize )
}


#[no_mangle]
pub extern "C" fn encl_end() -> bool {
    data().end()
}


#[no_mangle]
pub extern "C" fn encl_run(fid: u32, blk_nb: u32, buf: *const u8, len: usize) -> u32 {
    data().run( fid as usize, blk_nb, buf, len )    
}



pub fn decode<'a>( buf: *const u8,
                   len: usize,
                   out: &'a mut [u8]) -> usize {

    let (hdr,blk) = unsafe {
        let hdr = &*(buf as *const shared::Header);
        let blk_ptr = buf.offset( mem::size_of::<shared::Header>() as isize);
        let blk = slice::from_raw_parts( blk_ptr, hdr.size() as usize);
        (hdr,blk)
    };

    assert_eq!(blk.len()+mem::size_of::<shared::Header>(), len, "buffer size mismatch");

    // Decode de buffer.
    let res = rsgx_rijndael128GCM_decrypt( &shared::AES_KEY, blk, hdr.iv(), &[], hdr.mac(),  out );
    if let Err(err) = res {
        panic!("rsgx_rijndael128GCM_decrypt return {:?}", err );
    }

    hdr.nb_keys() as usize
}


