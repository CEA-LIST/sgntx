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


use std::path;
use std::fs;

use shared;

#[derive(Debug)]
pub struct Vcf {
    pub ec_path:  path::PathBuf,
    pub kind:     shared::Kind,
    pub size:     u64,
    pub blk_nb:   u32,
    pub fid:      u32,
    pub eof:      bool,
}

impl Vcf {
    pub fn new( ec_path: path::PathBuf, kind: shared::Kind, fid: u32 ) -> Vcf {
        let file = fs::File::open(&ec_path).unwrap();
        let size = file.metadata().unwrap().len();
        Vcf { ec_path: ec_path, kind: kind, size: size, blk_nb: 0, fid: fid , eof: false }
    }
}

