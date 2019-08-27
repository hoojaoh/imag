//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

use std::process::exit;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use failure::Error;
use ansi_term::Colour::Red;

struct ImagTrace<'a, T: 'a + ?Sized>(&'a T);

impl<'a, T: 'a + ?Sized> ImagTrace<'a, T> {
    fn new(d: &'a T) -> ImagTrace<'a, T> {
        ImagTrace(d)
    }
}

impl<'a> Display for ImagTrace<'a, Error>
{
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        writeln!(fmt, "{}: {}", Red.blink().paint("ERROR[   0]"), self.0)?;

        {
            for (i, cause) in self.0.iter_causes().enumerate() {
                writeln!(fmt,
                                 "{prefix}: {error}",
                                 prefix = Red.blink().paint(format!("ERROR[{:>4}]", i + 1)),
                                 error = cause)?;
            }
        }

        writeln!(fmt, "{}", Red.paint("--- BACKTRACE ---"))?;
        writeln!(fmt, "{:?}", self.0.backtrace())?;

        Ok(())
    }
}


pub fn trace_error(e: &Error) {
    eprintln!("{}", ImagTrace::new(e));
}

pub fn trace_error_dbg(e: &Error) {
    debug!("{}", ImagTrace::new(e));
}

/// Helper functions for `Result<T, E>` types to reduce overhead in the following situations:
///
/// ```ignore
/// function().map_err(|e| { trace_error(&e); e })
/// ```
///
/// and variants
pub trait MapErrTrace {
    type Output;

    fn map_err_trace(self) -> Self;
    fn map_err_trace_exit_unwrap(self) -> Self::Output;
}

impl<U> MapErrTrace for Result<U, Error> {
    type Output = U;

    /// Simply call `trace_error()` on the Err (if there is one) and return the error.
    ///
    /// This does nothing besides the side effect of printing the error trace
    fn map_err_trace(self) -> Self {
        self.map_err(|e| { trace_error(&e); e })
    }

    /// Trace the error and exit or unwrap the Ok(_).
    fn map_err_trace_exit_unwrap(self) -> Self::Output {
        self.map_err(|e| { trace_error(&e); exit(1) }).unwrap()
    }

}

