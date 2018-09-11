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


use core::fmt::{self,Write};
use ocall;

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe { ocall::ocall_print_string(s.as_ptr() as *const u8, s.len() ); }
        Result::Ok(())
    }    
}

//#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = console::Console.write_fmt(format_args!($($arg)*));
    })
}

//#[macro_export]
macro_rules! println {
    ($fmt:expr) => ( print!(concat!($fmt, "\n")) );
    ($fmt:expr, $($arg:tt)*) => ( print!(concat!($fmt, "\n"), $($arg)*) );
}


