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

// functions to ask the user for data, with crate:spinner

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

use regex::Regex;
use ansi_term::Colour::*;
use failure::Error;
use failure::ResultExt;
use failure::Fallible as Result;

/// Ask the user for a Yes/No answer. Optionally provide a default value. If none is provided, this
/// keeps loop{}ing
pub fn ask_bool(s: &str, default: Option<bool>, input: &mut dyn Read, output: &mut dyn Write) -> Result<bool> {
    ask_bool_(s, default, &mut BufReader::new(input), output)
}

fn ask_bool_<R: BufRead>(s: &str, default: Option<bool>, input: &mut R, output: &mut dyn Write) -> Result<bool> {
    lazy_static! {
        static ref R_YES: Regex = Regex::new(r"^[Yy](\n?)$").unwrap();
        static ref R_NO: Regex  = Regex::new(r"^[Nn](\n?)$").unwrap();
    }

    loop {
        ask_question(s, false, output)?;
        if match default { Some(s) => s, _ => true } {
            writeln!(output, " [Yn]: ")?;
        } else {
            writeln!(output, " [yN]: ")?;
        }

        let mut s = String::new();
        let _     = input.read_line(&mut s);

        if R_YES.is_match(&s[..]) {
            return Ok(true)
        } else if R_NO.is_match(&s[..]) {
            return Ok(false)
        } else if let Some(default) = default {
            return Ok(default)
        }
        // else again...
    }
}

/// Helper function to print a imag question string. The `question` argument may not contain a
/// trailing questionmark.
///
/// The `nl` parameter can be used to configure whether a newline character should be printed
pub fn ask_question(question: &str, nl: bool, output: &mut dyn Write) -> Result<()> {
    if nl {
        writeln!(output, "[imag]: {}?", Yellow.paint(question))
    } else {
        write!(output, "[imag]: {}?", Yellow.paint(question))
    }
    .context("Failed to write question to output")
    .map_err(Error::from)
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use super::ask_bool_;

    #[test]
    fn test_ask_bool_nodefault_yes() {
        let question = "Is this true";
        let default  = None;
        let answers  = "\n\n\n\n\ny";
        let mut sink: Vec<u8> = vec![];

        assert!(ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_nodefault_yes_nl() {
        let question = "Is this true";
        let default  = None;
        let answers  = "\n\n\n\n\ny\n";
        let mut sink: Vec<u8> = vec![];

        assert!(ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_nodefault_no() {
        let question = "Is this true";
        let default  = None;
        let answers  = "n";
        let mut sink: Vec<u8> = vec![];

        assert!(!ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_nodefault_no_nl() {
        let question = "Is this true";
        let default  = None;
        let answers  = "n\n";
        let mut sink: Vec<u8> = vec![];

        assert!(!ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_no() {
        let question = "Is this true";
        let default  = Some(false);
        let answers  = "n";
        let mut sink: Vec<u8> = vec![];

        assert!(!ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_no_nl() {
        let question = "Is this true";
        let default  = Some(false);
        let answers  = "n\n";
        let mut sink: Vec<u8> = vec![];

        assert!(!ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_yes() {
        let question = "Is this true";
        let default  = Some(true);
        let answers  = "y";
        let mut sink: Vec<u8> = vec![];

        assert!(ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_yes_nl() {
        let question = "Is this true";
        let default  = Some(true);
        let answers  = "y\n";
        let mut sink: Vec<u8> = vec![];

        assert!(ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_yes_answer_no() {
        let question = "Is this true";
        let default  = Some(true);
        let answers  = "n";
        let mut sink: Vec<u8> = vec![];

        assert!(!ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_no_answer_yes() {
        let question = "Is this true";
        let default  = Some(false);
        let answers  = "y";
        let mut sink: Vec<u8> = vec![];

        assert!(ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_no_without_answer() {
        let question = "Is this true";
        let default  = Some(false);
        let answers  = "\n";
        let mut sink: Vec<u8> = vec![];

        assert!(!ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_yes_without_answer() {
        let question = "Is this true";
        let default  = Some(true);
        let answers  = "\n";
        let mut sink: Vec<u8> = vec![];

        assert!(ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

}
