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

use std::fmt::Display;
use std::fmt::Debug;

use failure::Fail;

#[derive(Debug)]
pub enum Error {
    Io(::std::io::Error),
    Other(::failure::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Error::Io(e)    => write!(f, "{}", e),
            Error::Other(e) => write!(f, "{}", e),
        }
    }
}

impl Fail for Error {
    /* empty */
}

impl From<::std::io::Error> for Error {
    fn from(ioe: ::std::io::Error) -> Self {
        Error::Io(ioe)
    }
}

impl From<::failure::Error> for Error {
    fn from(fe: ::failure::Error) -> Self {
        Error::Other(fe)
    }
}

impl<D> From<::failure::Context<D>> for Error
    where D: Debug + Display + Send + Sync
{
    fn from(ctx: ::failure::Context<D>) -> Self {
        Error::Other(ctx.into())
    }
}

/// Convenient helper type
pub type Result<T> = ::std::result::Result<T, Error>;

