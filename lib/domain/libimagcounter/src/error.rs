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
        CounterError, CounterErrorKind, ResultExt, Result;
    }

    errors {
        StoreIdError            {
            description("StoreId error")
            display("StoreId error")
        }

        StoreReadError          {
            description("Store read error")
            display("Store read error")
        }

        StoreWriteError         {
            description("Store write error")
            display("Store write error")
        }

        HeaderTypeError         {
            description("Header type error")
            display("Header type error")
        }

        HeaderFieldMissingError {
            description("Header field missing error")
            display("Header field missing error")
        }

    }
}

pub use self::error::CounterError;
pub use self::error::CounterErrorKind;

impl IntoError for CounterErrorKind {
    type Target = CounterError;

    fn into_error(self) -> Self::Target {
        CounterError::from_kind(self)
    }

    fn into_error_with_cause(self, cause: Box<Error>) -> Self::Target {
        CounterError::from_kind(self)
    }
}
