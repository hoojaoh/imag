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

use std::path::PathBuf;

use failure::Fallible as Result;

use libimagrt::runtime::Runtime;
use libimagstore::storeid::StoreId;

pub fn delete(rt: &Runtime) -> Result<()> {
    let scmd  = rt.cli().subcommand_matches("delete").unwrap();
    let id    = scmd.value_of("id").unwrap(); // safe by clap
    let path  = PathBuf::from(id);
    let path  = StoreId::new(path)?;
    debug!("Deleting file at {:?}", id);

    rt.store().delete(path)
}

#[cfg(test)]
mod tests {
    use crate::create::create;
    use super::delete;

    use std::path::PathBuf;

    make_mock_app! {
        app "imag-store";
        modulename mock;
        version env!("CARGO_PKG_VERSION");
        with help "imag-store mocking app";
        with ui builder function crate::ui::build_ui;
    }
    use self::mock::generate_test_runtime;
    use self::mock::reset_test_runtime;

    #[test]
    fn test_delete_simple() {
        let test_name = "test_create_simple";
        let rt = generate_test_runtime(vec!["create", "test_create_simple"]).unwrap();

        create(&rt).unwrap();

        let rt = reset_test_runtime(vec!["delete", "test_create_simple"], rt).unwrap();

        delete(&rt).unwrap();

        let e = rt.store().get(PathBuf::from(test_name));
        assert!(e.is_ok());
        let e = e.unwrap();
        assert!(e.is_none());
    }

}

