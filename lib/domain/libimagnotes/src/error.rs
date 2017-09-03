//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
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

error_chain! {
    types {
        NoteError, NoteErrorKind, ResultExt, Result;
    }

    errors {
        StoreWriteError       {
            description("Error writing store")
            display("Error writing store")
        }

        StoreReadError        {
            description("Error reading store")
            display("Error reading store")
        }

        HeaderTypeError       {
            description("Header type error")
            display("Header type error")
        }

        NoteToEntryConversion {
            description("Error converting Note instance to Entry instance")
            display("Error converting Note instance to Entry instance")
        }

    }
}

pub use self::error::NoteError;
pub use self::error::NoteErrorKind;
pub use self::error::MapErrInto;

impl IntoError for NoteErrorKind {
    type Target = NoteError;

    fn into_error(self) -> Self::Target {
        NoteError::from_kind(self)
    }

    fn into_error_with_cause(self, cause: Box<Error>) -> Self::Target {
        NoteError::from_kind(self)
    }
}
