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


use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;

use core::cmp;
use core::fmt::Write;

use types::{Value,BlockInfo,Vcf};
use shared::{self,Kind,Key,KEY_MIN,KEY_MAX,Typ};
use ocall;
use hashmap;
use chisquare;
use console;
use spin::Mutex;



// To store the data between enclave calls.
pub struct GlobalData {
    // Files
    files:      Box<[Vcf]>,
    // nb_files:   usize,
    nb_control: f64,
    nb_case:    f64,
    // Buffer.
    buffers:        Mutex<Vec<Box<[Key]>>>,
    keys_per_block: usize, 
    // Keys.
    nb_keys:   u32,
    prev_key:  Key,
    // Our container.
    map:        hashmap::HashMap<Key,Value>,
    max_len:    usize,
    total_key:  usize,
    last_key:   Key,
    // Top Most
    top_most:   Vec<(Key,f64)>,
    output_allele_freq: bool
}



impl GlobalData {
    pub fn new( nb_control:         usize,
                nb_case:            usize,
                keys_per_block:     u32,
                nb_keys:            u32,
                snp_cnt:            usize,
                output_allele_freq: bool ) -> GlobalData {
        let nb_files = nb_control + nb_case;

        let mut files  = Vec::with_capacity( nb_files as usize);
        unsafe {
            files.set_len( nb_files );
        }
        
        GlobalData {
            // nb_files:       nb_files,
            nb_control:     nb_control as f64,
            nb_case:        nb_case as f64,
            files:          files.into_boxed_slice(),
            buffers:        Mutex::new( Vec::with_capacity( 8 ) ),
            keys_per_block: keys_per_block as usize,
            nb_keys:        nb_keys ,
            prev_key:       KEY_MIN,
            map:            hashmap::HashMap::new( nb_keys as usize * 34 ),
            max_len:        0,
            total_key:      0,
            last_key:       KEY_MAX,
            top_most:       Vec::with_capacity( snp_cnt ),
            output_allele_freq: output_allele_freq,
        }
    }
    
    pub fn register(&mut self, fid:  usize, kind: Kind) {
        self.files[fid as usize] = Vcf::new( kind );
    }

    pub fn begin(&mut self, fid:  usize) -> u32 {
        let key = self.prev_key;
        let vcf = &mut self.files[fid as usize];

        let blk_nb = {
            let iter = vcf.blocks.iter().filter(|&bi| bi.key <= key );
            match iter.last() {
                None => if vcf.last_key != KEY_MIN { 0 } else { 1 },
                Some(block_info) => block_info.blk_nb,
            }
        };

        vcf.clear();
        
        blk_nb
    }

    fn comp_and_output_af(&mut self) {
        let n = (self.nb_control + self.nb_case) * 2.0;

        // Sort SNP before writing allele file. 
        let mut vec: Vec<(Key,Value)> = self.map.iter().map(|(&k,&v)| (k,v)).collect();
        vec.sort_by(|a,b| a.0.cmp(&b.0) );
    
        let mut line_cnt = 0;
        let mut buff = String::new();
        for &(k,v) in vec.iter() {
            if line_cnt == 256 {
                unsafe {
                    ocall::ocall_append_file(buff.as_ptr() as *const u8, buff.len(), false);
                };
                line_cnt = 0;
                buff.clear();
            }
            let af = (v.0+v.1) as f64 / n;
            writeln!(&mut buff, "{}\t{}", k, af ).unwrap();
            line_cnt+=1;
        }

        if buff.is_empty() == false {
            unsafe {
                ocall::ocall_append_file(buff.as_ptr() as *const u8, buff.len(), false);
            };    
        }
    }
    
    fn output_top_snp(&self) {
        let mut buff = String::new();
        for &(k,chi2) in self.top_most.iter() {
            writeln!(&mut buff, "{}\t{:.14}", k, chisquare::chi2df3_sf(chi2) ).unwrap();
        }

        unsafe {
            ocall::ocall_append_file(buff.as_ptr() as *const u8, buff.len(), true);
        };
    }

    pub fn end(&mut self) -> bool {
        self.max_len = cmp::max( self.max_len, self.map.len() );
    
        // Compute chisquare.
        let n1 = self.nb_control * 2.0;
        let n2 = self.nb_case * 2.0;
        let n = n1+n2;

        for (&k,v) in self.map.iter() {
            let chi2 = chisquare::chisquare_stat(n, n1, n2, v.0 as f64, v.1 as f64);

            if self.top_most.len() < self.top_most.capacity() {
                self.top_most.push( (k,chi2) );
                if self.top_most.len() == self.top_most.capacity() {
                    self.top_most.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap().reverse() );
                }
            } else {
                if self.top_most.last().unwrap().1 < chi2 {
                    let _ = self.top_most.pop();
                    let pos =
                        match self.top_most.binary_search_by( |&(_,ch)| ch.partial_cmp(&chi2).unwrap().reverse() ) {
                            Ok(pos) => pos+1,
                            Err(pos) => pos,
                        };
                    self.top_most.insert(pos, (k,chi2) );
                }
            }
        }

        self.total_key += self.map.len();
    
        if self.output_allele_freq {
            self.comp_and_output_af();
        }

        if self.last_key != KEY_MAX {
            self.prev_key = self.last_key;
            self.last_key = KEY_MAX;
            // clear the list of the key.
            self.map.clear();
            true
        } else {
            println!("found {} keys", self.total_key );
            self.output_top_snp();
            false
        }
    }
    
    fn acquire_buffer(&mut self) -> Box<[Key]> {
        match self.buffers.lock().pop() {
            Some(buf) => buf,
            None      => {
                let mut vec = Vec::with_capacity( self.keys_per_block );
                unsafe { vec.set_len( self.keys_per_block ); }
                vec.into_boxed_slice()
            },
        }
    }

    fn release_buffer(&mut self, buf: Box<[Key]>) {
        self.buffers.lock().push( buf );
    }
    
    pub fn run(&mut self, fid: usize, blk_nb: u32, buf: *const u8, len: usize) -> u32 {
        let mut buffer = self.acquire_buffer();
        let mut end_flag = false;

        // Decrypt the block.
        let nb_keys = ::decode( buf, len, shared::as_u8_slice_mut( &mut buffer[..] ) );
        assert_ne!( nb_keys, 0 );

        {
            let blk = &mut buffer[..nb_keys];
            let vcf = &mut self.files[fid as usize];

            vcf.blocks.push( BlockInfo { blk_nb: blk_nb, key: blk[0] } );
    
            let pos = match blk.binary_search( &self.prev_key ) {
                Ok(pos)  => pos+1,
                Err(pos) => pos,
            };

            for &key in &blk[pos..] {

                let cnt = match key.typ() {
                    Typ::Heterozygous => 1, 
                    Typ::Homozygous   => 2,
                };

                if key <= self.last_key {
                    vcf.key_count += 1;

                    let mut v = self.map.insert( key );
                    v.update( vcf.kind, cnt );
            
                    if vcf.key_count == self.nb_keys && self.last_key == KEY_MAX {
                        self.last_key = key;
                        end_flag = true ;
                        break
                    }
                } else {
                    end_flag = true ;
                    break;
                }
            }
        }
    
        self.release_buffer( buffer );
    
        if end_flag { blk_nb } else { blk_nb+1 }
    }
}




