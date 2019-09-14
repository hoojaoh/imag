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

use runtime::Runtime;
use clap::App;
use failure::Fallible as Result;

pub trait ImagApplication {
    fn run(rt: Runtime) -> Result<()>;
    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a>;
    fn name() -> &'static str;
    fn version() -> &'static str;
    fn description() -> &'static str;
}


#[macro_export]
macro_rules! simple_imag_application_binary {
    ($application_library:ident, $application_implementor:ident) => {
        extern crate libimagerror;
        extern crate failure;
        extern crate $application_library;

        use failure::{Error, Fallible as Result};
        
        fn main() {
            use libimagerror::trace::MapErrTrace;
            use libimagrt::application::ImagApplication;
            use libimagrt::setup::generate_runtime_setup;
            use $application_library::$application_implementor;
            
            let version = make_imag_version!();
            let rt = generate_runtime_setup($application_implementor::name(),
                                            &version,
                                            $application_implementor::description(),
                                            $application_implementor::build_cli);

            // The error context must have a 'static lifetime
            // Therefore, the easiest, safe, but hacky way to achieve this
            // is to allocate a string, which is then forgotten to
            // leak memory and return it's contents as a &'static str
            // Because this is the very end of the application and only
            // happens once, it should have no impact whatsoever
            let error_context: &'static str = Box::leak(
                format!("Failed to run {}", $application_implementor::name())
                    .into_boxed_str()
            );
            $application_implementor::run(rt)
                .map_err(|e| e.context(error_context))
		.map_err(Error::from)
		.map_err_trace_exit_unwrap();
        }
    };
}
