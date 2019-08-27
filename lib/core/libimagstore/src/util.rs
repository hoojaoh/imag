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

use std::fmt::Write;

use toml::Value;
use failure::Fallible as Result;
use failure::ResultExt;

use libimagerror::errors::ErrorMsg as EM;

#[cfg(feature = "early-panic")]
#[macro_export]
macro_rules! if_cfg_panic {
    ()                       => { panic!() };
    ($msg:expr)              => { panic!($msg) };
    ($fmt:expr, $($arg:tt)+) => { panic!($fmt, $($arg),+) };
}

#[cfg(not(feature = "early-panic"))]
#[macro_export]
macro_rules! if_cfg_panic {
    ()                       => { };
    ($msg:expr)              => { };
    ($fmt:expr, $($arg:tt)+) => { };
}

pub fn entry_buffer_to_header_content(buf: &str) -> Result<(Value, String)> {

    debug!("Building entry from string");
    let mut header          = String::new();
    let mut content         = String::new();
    let mut header_consumed = false;

    let mut iter = buf.split('\n').skip(1).peekable(); // the first line is "---"

    while let Some(line) = iter.next() {
        if line == "---" && !header_consumed {
            header_consumed = true;
            // do not further process the line
        } else if !header_consumed {
            writeln!(header, "{}", line).context(EM::FormatError)?;
        } else if iter.peek().is_some() {
            writeln!(content, "{}", line).context(EM::FormatError)?;
        } else {
            write!(content, "{}", line).context(EM::FormatError)?;
        }
    }

    let h = ::toml::de::from_str(&header).context(EM::TomlDeserError)?;
    Ok((h, content))
}

#[cfg(test)]
mod test {
    extern crate env_logger;

    use super::entry_buffer_to_header_content;

    fn setup_logging() {
        let _ = env_logger::try_init();
    }

    fn mkfile(content: &str) -> String {
        format!(r#"---
[imag]
version = '{version}'
---
{content}"#, version = env!("CARGO_PKG_VERSION"), content = content)
    }

    #[test]
    fn test_entry_buffer_to_header_content_1() {
        let content = "Hai";
        let file = format!(r#"---
[imag]
version = '{version}'
---
{content}"#, version = env!("CARGO_PKG_VERSION"), content = content);

        let res = entry_buffer_to_header_content(&file);

        assert!(res.is_ok());
        let (_, res_content) = res.unwrap();
        assert_eq!(res_content, content)
    }

    #[test]
    fn test_entry_buffer_to_header_content_2() {
        setup_logging();
        let content = r#"Hai
"#;

        let file = mkfile(&content);
        debug!("FILE: <<<{}>>>", file);
        let res  = entry_buffer_to_header_content(&file);

        assert!(res.is_ok());
        let (_, res_content) = res.unwrap();
        debug!("CONTENT: <<<{}>>>", res_content);
        assert_eq!(res_content, content)
    }

    #[test]
    fn test_entry_buffer_to_header_content_3() {
        let content = r#"Hai

barbar

"#;

        let file = mkfile(&content);
        let res  = entry_buffer_to_header_content(&file);

        assert!(res.is_ok());
        let (_, res_content) = res.unwrap();
        assert_eq!(res_content, content)
    }

    #[test]
    fn test_entry_buffer_to_header_content_4() {
        let content = r#"Hai

            ---
barbar
            ---

"#;

        let file = mkfile(&content);
        let res  = entry_buffer_to_header_content(&file);

        assert!(res.is_ok());
        let (_, res_content) = res.unwrap();
        assert_eq!(res_content, content)
    }

    #[test]
    fn test_entry_buffer_to_header_content_5() {
        let content = r#"Hai

---
barbar
---


"#;

        let file = mkfile(&content);
        let res  = entry_buffer_to_header_content(&file);

        assert!(res.is_ok());
        let (_, res_content) = res.unwrap();
        assert_eq!(res_content, content)
    }

}

