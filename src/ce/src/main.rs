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


extern crate clap;
use clap::{Arg, App};


extern crate walkdir;
use walkdir::{WalkDir, WalkDirIterator};

extern crate rayon;
use rayon::prelude::*;

extern crate openssl_sys;

extern crate rand;

extern crate shared;

use std::path::PathBuf;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant};
use std::fs;

mod compress;

fn main() {
    let start = Instant::now();
    println!("-=< Compression & Encryption >=-");
    
    // Read command line arguments.
    let matches =
        App::new("ce")
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
        .arg(Arg::with_name("out_path")
             .help("Output directory")
             .short("o")
             .long("out_path")
             .value_name("STR")
             .required(false)
             .default_value("")
             .takes_value(true))          
        .get_matches();

    // This is safe because args control and case are required.
    let (control,case) = (matches.value_of("control").unwrap(), matches.value_of("case").unwrap());

    let out_path = matches.value_of("out_path").unwrap();

    let (out_path_cont, out_path_case) = {
        if out_path.len() == 0 {
            (PathBuf::from("out/control"), PathBuf::from("out/case"))
        } else {
            let mut t1 = PathBuf::from(out_path);
            t1.push("control");
            let mut t2 = PathBuf::from(out_path);
            t2.push("case");
            (t1,t2)
        }
    };

    // Found control files.
    let mut controls: Vec<_> = WalkDir::new( control )
        .min_depth( 1 )
        .follow_links( true )
        .into_iter()
        .filter_entry(|e| e.path().extension().unwrap() == "vcf")
        .map( |e| {
            let path = e.unwrap().path().to_path_buf();
            let mut to_path = out_path_cont.clone();
            to_path.push( &path.file_stem().unwrap() );
            to_path.set_extension( "ce" );
            let _ = fs::remove_file( &to_path );
            (path, to_path)
        }).collect();
    println!("Control path {}: found {} vcf files", control, controls.len() );

    // Found case files.
    let mut cases: Vec<_> = WalkDir::new( case )
        .min_depth(1)
        .follow_links( true )
        .into_iter()
        .filter_entry(|e| e.path().extension().unwrap() == "vcf")
        .map( |e| {
            let path = e.unwrap().path().to_path_buf();
            let mut to_path = out_path_case.clone();
            to_path.push( &path.file_stem().unwrap() );
            to_path.set_extension( "ce" );
            let _ = fs::remove_file( &to_path );
            (path, to_path)
        }).collect();
    println!("Case path {}: found {} vcf files", case, cases.len() );

    println!("Compressed and encrypted files output path: ({},{})",
             out_path_cont.to_str().unwrap(), out_path_case.to_str().unwrap());

    fs::create_dir_all(&out_path_cont).unwrap();
    fs::create_dir_all(&out_path_case).unwrap();

    // Compress and encrypt all files using parallele iterator.
    rayon::initialize(rayon::Configuration::new().num_threads(4)).unwrap();
    let nb = AtomicUsize::new( 1 );
    controls.par_iter_mut().chain( cases.par_iter_mut() )
        .for_each(|e| {
            print!("{:8}\r", nb.fetch_add( 1, Ordering::Relaxed ) );
            std::io::stdout().flush().unwrap();
            
            if let Err(why) = compress::compress(&e.0, &e.1, shared::KEYS_PER_BLOCK_DEFAULT) {
                panic!("{}", why);
            }
        });

    let dur = start.elapsed();
    let secs = dur.as_secs();
    println!("\nexecution time: {}m{:02}.{:03}", (secs/60), (secs%60), dur.subsec_nanos()/1_000_000 );
}
