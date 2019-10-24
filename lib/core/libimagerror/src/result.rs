//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 the imag contributors
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

/// Extension trait for doing `Result<Option<T>, E>  ->  Result<T, E>`
pub trait ResultOptionExt<T, E, F>
    where T: Sized,
          E: Sized,
          F: FnOnce() -> E
{
    fn inner_ok_or_else(self, f: F) -> Result<T, E>;
}

impl<T, E, F> ResultOptionExt<T, E, F> for Result<Option<T>, E>
    where T: Sized,
          E: Sized,
          F: FnOnce() -> E
{
    fn inner_ok_or_else(self, f: F) -> Result<T, E> {
        self.and_then(|opt| opt.ok_or_else(f))
    }
}

