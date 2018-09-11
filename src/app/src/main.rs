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


extern crate sgx_types;
use sgx_types::sgx_status_t::{SGX_SUCCESS};

extern crate clap;
use clap::{Arg, App};

extern crate walkdir;
use walkdir::{WalkDir, WalkDirIterator};

extern crate rayon;
use rayon::prelude::*;

extern crate shared;

use std::path;
use std::fs;
use std::io::{self,Read,Write,Seek,SeekFrom};
use std::ptr;
use std::ffi::CString;
use std::slice;
use std::u32;
use std::time::{Instant};

mod vcf;

fn read_token(path: &path::PathBuf) -> io::Result<sgx_types::sgx_launch_token_t> {
    let mut token: sgx_types::sgx_launch_token_t = [0;1024];
    // Try to read the token from the file.
    if path.exists() {
        // Open file
        let mut file = try!( fs::File::open(path) );
        // Read the token.
        try!( file.read_exact( &mut token ) );
    }
    // Return the token.
    Ok(token)
}


fn write_token(path: &path::PathBuf, token: &sgx_types::sgx_launch_token_t) -> io::Result<()> {
    let mut file = try!( fs::File::create(path) );
    try!( file.write( token ) );
    Ok(())
}


fn create_enclave( token_name: &str, enclave_name: &str ) -> sgx_types::sgx_enclave_id_t {
    // Make enclave name C string compatible.
    let enclave = match CString::new( enclave_name ) {
        Ok(e) => e,
        Err(why) => panic!("{}: {}", enclave_name, why ),
    };
    // Full token filename
    // let mut token_path = match env::home_dir() {
    //     Some(path) => path,
    //     None => path::PathBuf::new(),
    // };
    let mut token_path = path::PathBuf::new();
    token_path.push( token_name );
    // println!("token path = {}", token_path.display() );

    // Retrieve token value
    let mut token = match read_token( &token_path ) {
        Ok(token) => token,
        Err(why) => panic!("{}: {}", token_path.display(), why)
    };

    // Create the enclave.
    let mut updated = 0;
    let mut enclave_id = 0;
    let res = unsafe {
        sgx_types::sgx_create_enclave(enclave.as_ptr() as *const i8,
                                      1,
                                      &mut token,
                                      &mut updated,
                                      &mut enclave_id,
                                      ptr::null_mut())
    };
    if res != SGX_SUCCESS {
        panic!("ERROR: sgx_create_enclave returned {:?}", res );
    }
    // println!("enclave {} launched", enclave_name );

    // Update the token if needed
    if updated != 0 {
        println!("Updating {}", token_path.display() );
        if let Err(why) = write_token( &token_path, &token ) {
            panic!("{}: {}", token_path.display(), why)
        }
    }

    enclave_id
}


#[no_mangle]
pub extern "C" fn ocall_print_string(ptr: *const u8, len: usize) {
    let s = unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        std::str::from_utf8( slice )
    };
    print!("{}", s.unwrap_or("ERROR: invalid utf8 string") );
}

#[no_mangle]
pub extern "C" fn ocall_append_file(ptr: *const u8, len: usize, chisq_file: bool) {
    let filename = { 
        if chisq_file {
            app_params().chisq_file_name.as_str()
        } else {
            app_params().af_file_name.as_str()
        }
    };

    let buff_str = unsafe {
        std::str::from_utf8( slice::from_raw_parts(ptr, len) )
    }.unwrap_or("ERROR: invalid utf8 string");

    let file = fs::OpenOptions::new()
                            .append(true)
                            .open(filename);

    match file {
        Ok(mut file) => {
            file.write_all(buff_str.as_bytes()).unwrap();
        },
        Err(_) => panic!("ERROR: appending to file {}", filename),
    }
}

fn create_chisq_file() {
    let file = fs::OpenOptions::new()
                            .create(true)
                            .truncate(true)
                            .write(true)
                            .open(app_params().chisq_file_name.as_str());
    match file {
        Ok(mut file) => {
            writeln!(&mut file, "#Top most significant SNPs(sorted)").unwrap();
            writeln!(&mut file, "#CHROM\tPOS\tID\tREF\tALT\tp-value").unwrap();
        },
        Err(_) => panic!("ERROR: creating chisq output file"),
    }
}

fn create_af_file() {
    let file = fs::OpenOptions::new()
                            .create(true)
                            .truncate(true)
                            .write(true)
                            .open(app_params().af_file_name.as_str());
    match file {
        Ok(mut file) => {
            writeln!(&mut file, "#Allele frequecies of SNPs from two groups").unwrap();
            writeln!(&mut file, "#CHROM\tPOS\tID\tREF\tALT\talleleFreq").unwrap();
        },
        Err(_) => panic!("ERROR: creating allele frequecies output file"),
    }
}


extern "C" {
    fn encl_init(eid:            sgx_types::sgx_enclave_id_t,
                 nb_control:     u32,
                 nb_case:        u32,
                 keys_per_block: u32,
                 nb_keys:        u32,
                 snp_cnt:        usize,
                 output_allele_freq: bool) -> sgx_types::sgx_status_t;
    
    fn encl_register(eid:     sgx_types::sgx_enclave_id_t,
                     fid:     u32,
                     kind:    shared::Kind ) -> sgx_types::sgx_status_t;

    fn encl_begin(eid:    sgx_types::sgx_enclave_id_t,
                  retval: *mut u32,
                  fid:    u32) -> sgx_types::sgx_status_t;
    
    fn encl_run(eid:    sgx_types::sgx_enclave_id_t,
                retval: *mut u32,
                fid:    u32,
                blk_nb: u32,
                blk:    *const u8,
                len:    usize) -> sgx_types::sgx_status_t;

    fn encl_end(eid: sgx_types::sgx_enclave_id_t, cont: *mut bool) -> sgx_types::sgx_status_t;
}



fn register_file( eid: sgx_types::sgx_enclave_id_t, vcf: &mut vcf::Vcf) {
    // Register the file.
    let res = unsafe { encl_register( eid, vcf.fid, vcf.kind ) };
    if res != SGX_SUCCESS {
         panic!("ERROR: encl_register returned {:?}", res);
    }
}


fn begin_loop( eid: sgx_types::sgx_enclave_id_t, vcf: &mut vcf::Vcf ) {
    let mut blk_nb = u32::MAX;
    let res = unsafe { encl_begin( eid, &mut blk_nb, vcf.fid ) };
    if res != SGX_SUCCESS {
        panic!("[{}]:{}:ERROR: encl_begin returned {:?}",
               vcf.ec_path.display(), vcf.blk_nb, res);
    }

    if blk_nb == 0 {
        vcf.eof = true;
    } else {
        vcf.blk_nb = blk_nb;
    }
}


fn read_file( eid: sgx_types::sgx_enclave_id_t, vcf: &mut vcf::Vcf ) -> io::Result<()> {
    assert_eq!( vcf.ec_path.is_file(), true );

    let mut blk_nb = vcf.blk_nb;
    
    // Open file and go to the last pos.
    let mut file = try!( fs::File::open(&vcf.ec_path) );
    let keys_per_block = app_params().keys_per_block;
    
    let pos = shared::block_size( keys_per_block ) as u64 * (blk_nb-1) as u64 ;
    assert_eq!( try!( file.seek( SeekFrom::Start(pos) ) ), pos );    

    let mut buffer = {
        let size = shared::block_size( keys_per_block ) as usize;
        let mut b = Vec::with_capacity( size );
        unsafe { b.set_len( size ) };
        b.into_boxed_slice()
    };
    
    loop {
        let readed = try!( file.read( &mut buffer ) );
        if readed == 0 { 
            break
        }

        let res = unsafe { encl_run( eid, &mut blk_nb, vcf.fid, vcf.blk_nb, buffer[..readed].as_ptr(), readed ) };

        if res != SGX_SUCCESS {
            panic!("[{}]:{}:ERROR: encl_run returned {:?}",
                   vcf.ec_path.display(), vcf.blk_nb, res);
        }
        
        if blk_nb == vcf.blk_nb { break; }
        vcf.blk_nb = blk_nb;
    };

    Ok(())
}


fn end_loop( eid: sgx_types::sgx_enclave_id_t ) -> bool {
    let mut cont = true ;
    let res = unsafe { encl_end( eid, &mut cont ) };
    if res != SGX_SUCCESS {
        panic!("ERROR: encl_end returned {:?}", res );
    }
    cont
}

#[derive(Debug)]
struct AppParams {
    control: String,
    case: String,
    snp_cnt: usize,
    chisq_file_name: String,
    af_file_name: String,
    output_allele_freq: bool,
    keys_per_block: u32,
    keys_per_iter: u32,
}

static mut APP_PARAMS: Option<AppParams> = None;

fn app_params() -> &'static mut AppParams {
    unsafe { APP_PARAMS.as_mut().unwrap() }
}

fn parse_cmd_args() -> AppParams {
    // Read command line arguments.
    let matches =
        App::new("app")
        .version("0.1")
        .arg(Arg::with_name("control")
             .help("Control .vcf directory")
             .short("C")
             .long("control")
             .value_name("DIR")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("case")
             .help("Case .vcf directory")
             .short("c")
             .long("case")
             .value_name("DIR")
             .required(true)
             .takes_value(true)) 
        .arg(Arg::with_name("snp_count")
             .help("Count of top SNP alleles to compute")
             .short("k")
             .long("snp_count")
             .value_name("INT")
             .required(false)
             .default_value("10")
             .takes_value(true))
        .arg(Arg::with_name("output")
             .help("Prefix of output files")
             .short("f")
             .long("output")
             .value_name("STR")
             .required(false)
             .default_value("")
             .takes_value(true))        
        .arg(Arg::with_name("output_allele_freq")
             .help("Output allele frequecies")
             .short("a")
             .long("output_allele_freq")
             .value_name("BOOL")
             .required(false)
             .takes_value(false))        
        .get_matches();

    let out_prefix = matches.value_of("output").unwrap().to_string();
    let mut chisq_file_name = out_prefix.clone();
    chisq_file_name.push_str("Chisq.vcf");
    let mut af_file_name = out_prefix.clone();
    af_file_name.push_str("AF.vcf");

    AppParams {
        control: matches.value_of("control").unwrap().to_string(),
        case: matches.value_of("case").unwrap().to_string(),
        snp_cnt: matches.value_of("snp_count").unwrap().parse::<usize>().unwrap(),
        chisq_file_name: chisq_file_name,
        af_file_name: af_file_name,
        output_allele_freq: matches.is_present("output_allele_freq"), 
        keys_per_block: shared::KEYS_PER_BLOCK_DEFAULT,  // keys_per_block,
        keys_per_iter: shared::KEYS_PER_BLOCK_DEFAULT * shared::ITER_FACTOR_DEFAULT + 1,
    }
}


fn run(eid: sgx_types::sgx_enclave_id_t, mut controls: Vec<vcf::Vcf>, mut cases: Vec<vcf::Vcf> ) {
    // Append and sort files in decreasing size order.
    controls.append( &mut cases );
    controls.sort_by(|a,b| b.size.cmp(&a.size) );

    // Read the files.
    let mut cont = true;
    let mut nb   = 1 ;
    while cont {
        print!("\t{}\r", nb );
        std::io::stdout().flush().unwrap();
    
        // Compute the next block to read
        for vcf in controls.iter_mut() {
            if !vcf.eof {
                begin_loop( eid, vcf );
            }
        }

        // Read the first file alone. 
        if let Some(vcf) = controls.first_mut() {
            if !vcf.eof {
                if let Err(why) = read_file( eid, vcf ) {
                    panic!("{}", why);
                }
            }
        }
        
        // Read the blocks
        controls.par_iter_mut().skip( 1 )
            .for_each(|vcf| {
                if !vcf.eof {
                    if let Err(why) = read_file( eid, vcf ) {
                        panic!("{}", why);
                    }
                }
            });

        cont = end_loop( eid );
        nb += 1 ;
    }
}


fn main() {
    let start = Instant::now();
    println!("-=< Analysing >=-");
    
    unsafe {
        APP_PARAMS = Some(parse_cmd_args());
    }
    let params = app_params();

    let mut fid: u32 = 0;

    // Found control files.
    let mut controls: Vec<_> = WalkDir::new( &params.control )
        .min_depth( 1 )
        .follow_links( true )
        .into_iter()
        .filter_entry(|e| e.path().extension().unwrap() == "ce")
        .map( |e| {
            let vcf = vcf::Vcf::new( e.unwrap().path().to_path_buf(),
                            shared::Kind::Control, fid );
            fid+=1;
            vcf
        }).collect();

    // Found case files.
    let mut cases: Vec<_> = WalkDir::new( &params.case )
        .min_depth(1)
        .follow_links( true )
        .into_iter()
        .filter_entry(|e| e.path().extension().unwrap() == "ce")
        .map( |e| {
            let vcf = vcf::Vcf::new( e.unwrap().path().to_path_buf(),
                            shared::Kind::Case, fid );
            fid+=1;
            vcf
        }).collect();
    
    // create chisquare and allele frequecies files
    create_chisq_file();
    if params.output_allele_freq {
        create_af_file();
    }

    // Create the enclave.
    let enclave_id = create_enclave( "enclave.token", "enclave.signed.so" );

    // Init the enclave with the key, the number of files, and the size of the buffer.
    let res = unsafe { encl_init( enclave_id,
                                  controls.len() as u32,
                                  cases.len() as u32,
                                  params.keys_per_block,
                                  params.keys_per_iter,
                                  params.snp_cnt,
                                  params.output_allele_freq ) };
    if res != SGX_SUCCESS {
        panic!("ERROR: encl_init returned {:?}", res );
    }

    println!("{}: Found {} .ce files", &params.control, controls.len() );
    println!("{}: Found {} .ce files", &params.case, cases.len() );
        
    // Register all files
    for vcf in controls.iter_mut().chain( cases.iter_mut() ) {
        register_file( enclave_id, vcf );
    }

    run( enclave_id, controls, cases );
    
    // Destroy the enclave
    unsafe { sgx_types::sgx_destroy_enclave( enclave_id ) };

    if params.output_allele_freq {
        println!("See files {} and {} for results", params.chisq_file_name, params.af_file_name );
    } else {
        println!("See file {} for result", params.chisq_file_name );
    }

    let dur = start.elapsed();
    let secs = dur.as_secs();
    println!("execution time: {}m{:02}.{:03}", (secs/60), (secs%60), dur.subsec_nanos()/1_000_000 );    
}

