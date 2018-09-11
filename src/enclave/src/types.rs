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


use core::fmt;
use alloc::vec::Vec;

use shared::{Kind,Key,KEY_MIN};


#[derive(Clone,Copy,Debug)]
pub struct Value(pub u32, pub u32);

impl Value {
    pub fn update(&mut self, kind: Kind, cnt: u32) {
        match kind {
            Kind::Control => self.0 += cnt,
            Kind::Case    => self.1 += cnt,
        }
    }
}

impl Default for Value {
    fn default() -> Value {
        Value(0,0)
    }
}



pub struct BlockInfo {
    pub key:     Key,
    pub blk_nb:  u32,
}


impl fmt::Debug for BlockInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}:{}]", self.blk_nb, self.key )
    }
}

pub struct Vcf {
    pub blocks:    Vec<BlockInfo>,
    pub kind:      Kind,
    pub last_key:  Key,
    pub key_count: u32,
}

impl Vcf {
    pub fn new(kind: Kind) -> Vcf {
        Vcf { kind: kind, blocks: Vec::new(), last_key: KEY_MIN, key_count: 0 }
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
        self.key_count = 0 ;
    }
}

