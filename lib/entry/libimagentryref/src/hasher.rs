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

use std::path::Path;

use failure::Fallible as Result;

pub trait Hasher {
    const NAME: &'static str;

    /// hash the file at path `path`
    fn hash<P: AsRef<Path>>(path: P) -> Result<String>;
}

pub mod default {
    pub use super::sha1::Sha1Hasher as DefaultHasher;
}

pub mod sha1 {
    use std::path::Path;

    use failure::Fallible as Result;
    use sha1::{Sha1, Digest};

    use crate::hasher::Hasher;

    pub struct Sha1Hasher;

    impl Sha1Hasher {
        pub fn sha1_hash(s: &str) -> String {
            format!("{:x}", Sha1::digest(s.as_bytes())) // TODO: Ugh...
        }
    }

    impl Hasher for Sha1Hasher {
        const NAME : &'static str = "sha1";

        fn hash<P: AsRef<Path>>(path: P) -> Result<String> {
            Ok(Sha1Hasher::sha1_hash(&::std::fs::read_to_string(path)?))
        }
    }

}

