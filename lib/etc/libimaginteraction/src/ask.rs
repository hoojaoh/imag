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
use std::result::Result as RResult;
use std::io::Read;
use std::io::Write;

use regex::Regex;
use ansi_term::Colour::*;
use interactor::*;
use failure::Error;
use failure::ResultExt;
use failure::Fallible as Result;
use failure::err_msg;

/// Ask the user for a Yes/No answer. Optionally provide a default value. If none is provided, this
/// keeps loop{}ing
pub fn ask_bool(s: &str, default: Option<bool>, input: &mut Read, output: &mut Write) -> Result<bool> {
    ask_bool_(s, default, &mut BufReader::new(input), output)
}

fn ask_bool_<R: BufRead>(s: &str, default: Option<bool>, input: &mut R, output: &mut Write) -> Result<bool> {
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
        } else if default.is_some() {
            return Ok(default.unwrap())
        }
        // else again...
    }
}

/// Ask the user for an unsigned number. Optionally provide a default value. If none is provided,
/// this keeps loop{}ing
pub fn ask_uint(s: &str, default: Option<u64>, input: &mut Read, output: &mut Write) -> Result<u64> {
    ask_uint_(s, default, &mut BufReader::new(input), output)
}

fn ask_uint_<R: BufRead>(s: &str, default: Option<u64>, input: &mut R, output: &mut Write) -> Result<u64> {
    use std::str::FromStr;

    loop {
        ask_question(s, false, output)?;

        let mut s = String::new();
        let _     = input.read_line(&mut s);

        let u : RResult<u64, _> = FromStr::from_str(&s[..]);
        match u {
            Ok(u)  => { return Ok(u); },
            Err(_) => {
                if default.is_some() {
                    return Ok(default.unwrap());
                } // else keep looping
            }
        }
    }
}

/// Ask the user for a String.
///
/// If `permit_empty` is set to false, the default value will be returned if the user inserts an
/// empty string.
///
/// If the `permit_empty` value is true, the `default` value is never returned.
///
/// If the `permit_multiline` is set to true, the `prompt` will be displayed before each input line.
///
/// If the `eof` parameter is `None`, the input ends as soon as there is an empty line input from
/// the user. If the parameter is `Some(text)`, the input ends if the input line is equal to `text`.
pub fn ask_string(s: &str,
                  default: Option<String>,
                  permit_empty: bool,
                  permit_multiline: bool,
                  eof: Option<&str>,
                  prompt: &str,
                  input: &mut Read,
                  output: &mut Write)
    -> Result<String>
{
    ask_string_(s,
                default,
                permit_empty,
                permit_multiline,
                eof,
                prompt,
                &mut BufReader::new(input),
                output)
}

fn ask_string_<R: BufRead>(s: &str,
                           default: Option<String>,
                           permit_empty: bool,
                           permit_multiline: bool,
                           eof: Option<&str>,
                           prompt: &str,
                           input: &mut R,
                           output: &mut Write)
    -> Result<String>
{
    let mut v = vec![];
    loop {
        ask_question(s, true, output)?;
        write!(output, "{}", prompt)?;

        let mut s = String::new();
        let _     = input.read_line(&mut s);

        if permit_multiline {
            if permit_multiline && eof.map_or(false, |e| e == s) {
                return Ok(v.join("\n"));
            }

            if permit_empty || !v.is_empty() {
                v.push(s);
            }
            write!(output, "{}", prompt)?;
        } else if s.is_empty() && permit_empty {
            return Ok(s);
        } else if s.is_empty() && !permit_empty {
            if default.is_some() {
                return Ok(default.unwrap());
            } else {
                continue;
            }
        } else {
            return Ok(s);
        }
    }
}

pub fn ask_select_from_list(list: &[&str]) -> Result<String> {
    pick_from_list(default_menu_cmd().as_mut(), list, "Selection: ")
        .context(err_msg("Unknown interaction error"))
        .map_err(Error::from)
}

/// Helper function to print a imag question string. The `question` argument may not contain a
/// trailing questionmark.
///
/// The `nl` parameter can be used to configure whether a newline character should be printed
pub fn ask_question(question: &str, nl: bool, output: &mut Write) -> Result<()> {
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
    use super::ask_uint_;

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

        assert!(false == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_nodefault_no_nl() {
        let question = "Is this true";
        let default  = None;
        let answers  = "n\n";
        let mut sink: Vec<u8> = vec![];

        assert!(false == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_no() {
        let question = "Is this true";
        let default  = Some(false);
        let answers  = "n";
        let mut sink: Vec<u8> = vec![];

        assert!(false == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_no_nl() {
        let question = "Is this true";
        let default  = Some(false);
        let answers  = "n\n";
        let mut sink: Vec<u8> = vec![];

        assert!(false == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_yes() {
        let question = "Is this true";
        let default  = Some(true);
        let answers  = "y";
        let mut sink: Vec<u8> = vec![];

        assert!(true == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_yes_nl() {
        let question = "Is this true";
        let default  = Some(true);
        let answers  = "y\n";
        let mut sink: Vec<u8> = vec![];

        assert!(true == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_yes_answer_no() {
        let question = "Is this true";
        let default  = Some(true);
        let answers  = "n";
        let mut sink: Vec<u8> = vec![];

        assert!(false == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_no_answer_yes() {
        let question = "Is this true";
        let default  = Some(false);
        let answers  = "y";
        let mut sink: Vec<u8> = vec![];

        assert!(true == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_no_without_answer() {
        let question = "Is this true";
        let default  = Some(false);
        let answers  = "\n";
        let mut sink: Vec<u8> = vec![];

        assert!(false == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_bool_default_yes_without_answer() {
        let question = "Is this true";
        let default  = Some(true);
        let answers  = "\n";
        let mut sink: Vec<u8> = vec![];

        assert!(true == ask_bool_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_uint_nodefault() {
        let question = "Is this 1";
        let default  = None;
        let answers  = "1";
        let mut sink: Vec<u8> = vec![];

        assert!(1 == ask_uint_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_uint_default() {
        let question = "Is this 1";
        let default  = Some(1);
        let answers  = "1";
        let mut sink: Vec<u8> = vec![];

        assert!(1 == ask_uint_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_uint_default_2_input_1() {
        let question = "Is this 1";
        let default  = Some(2);
        let answers  = "1";
        let mut sink: Vec<u8> = vec![];

        assert!(1 == ask_uint_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_uint_default_2_noinput() {
        let question = "Is this 1";
        let default  = Some(2);
        let answers  = "\n";
        let mut sink: Vec<u8> = vec![];

        assert!(2 == ask_uint_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_uint_default_2_several_noinput() {
        let question = "Is this 1";
        let default  = Some(2);
        let answers  = "\n\n\n\n";
        let mut sink: Vec<u8> = vec![];

        assert!(2 == ask_uint_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

    #[test]
    fn test_ask_uint_default_2_wrong_input() {
        let question = "Is this 1";
        let default  = Some(2);
        let answers  = "\n\n\nasfb\nsakjf\naskjf\n-2";
        let mut sink: Vec<u8> = vec![];

        assert!(2 == ask_uint_(question, default, &mut BufReader::new(answers.as_bytes()), &mut sink).unwrap());
    }

}
