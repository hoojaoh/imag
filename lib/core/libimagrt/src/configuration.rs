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

use toml::Value;
use clap::App;
use failure::ResultExt;
use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;

use libimagerror::errors::ErrorMsg as EM;

/// Get a new configuration object.
///
/// The passed runtimepath is used for searching the configuration file, whereas several file
/// names are tested. If that does not work, the home directory and the XDG basedir are tested
/// with all variants.
pub fn fetch_config(searchpath: &PathBuf) -> Result<Option<Value>> {
    use std::env;
    use std::fs::File;
    use std::io::Read;
    use std::io::Write;
    use std::io::stderr;

    use libimagutil::variants::generate_variants as gen_vars;
    use libimagerror::trace::trace_error;

    let variants : Vec<&'static str> = vec!["config", "config.toml", "imagrc", "imagrc.toml"];
    let modifier = |base: &PathBuf, v: &&str| {
        let mut base = base.clone();
        base.push(String::from(*v));
        base
    };

    let vals = vec![
        vec![searchpath.clone()],
        gen_vars(searchpath, variants.iter(), &modifier),

        env::var("HOME")
            .map(|home| gen_vars(&PathBuf::from(home), variants.iter(), &modifier))
            .unwrap_or(vec![]),

        xdg_basedir::get_data_home()
            .map(|data_dir| gen_vars(&data_dir, variants.iter(), &modifier))
            .unwrap_or(vec![]),
    ];

    let config = vals
        .iter()
        .flatten()
        .filter(|path| path.exists() && path.is_file())
        .filter_map(|path| {
            let content = {
                let f = File::open(path);
                if f.is_err() {
                    let _ = write!(stderr(), "Error opening file: {:?}", f);
                    return None
                }
                let mut f = f.unwrap();

                let mut s = String::new();
                f.read_to_string(&mut s).ok();
                s
            };

            ::toml::de::from_str::<::toml::Value>(&content[..])
                .map(Some)
                .unwrap_or_else(|e| {
                    let line_col = e
                        .line_col()
                        .map(|(line, col)| format!("Line {}, Column {}", line, col))
                        .unwrap_or_else(|| String::from("Line unknown, Column unknown"));

                    let _ = write!(stderr(), "Config file parser error at {}", line_col);
                    let e = Error::from(EM::TomlDeserError);
                    trace_error(&e);
                    None
                })
        })
        .nth(0);

    Ok(config)
}

/// Override the configuration.
/// The `v` parameter is expected to contain 'key=value' pairs where the key is a path in the
/// TOML tree, the value to be an appropriate value.
///
/// The override fails if the configuration which is about to be overridden does not exist or
/// the `value` part cannot be converted to the type of the configuration value.
///
/// If `v` is empty, this is considered to be a successful `override_config()` call.
pub fn override_config(val: &mut Value, v: Vec<String>) -> Result<()> {
    use libimagutil::key_value_split::*;
    use toml_query::read::TomlValueReadExt;

    v.into_iter()
        .map(|s| { debug!("Trying to process '{}'", s); s })
        .filter_map(|s| s.into_kv().map(Into::into).or_else(|| {
            warn!("Could split at '=' - will be ignore override");
            None
        }))
        .map(|(k, v)| {
            let value = val.read_mut(&k)
                .context(EM::TomlQueryError)?
                .ok_or_else(|| Error::from(err_msg("No config value there, cannot override.")))?;

            let new_value = into_value(value, v)
                .ok_or_else(|| Error::from(err_msg("Config override type not matching")))?;

            info!("Successfully overridden: {} = {}", k, new_value);
            *value = new_value;
            Ok(())
        })
        .map(|elem: Result<()>| elem.context(err_msg("Config override error")).map_err(Error::from))
        .collect::<Result<()>>()
}

/// Tries to convert the String `s` into the same type as `value`.
///
/// Returns None if string cannot be converted.
///
/// Arrays and Tables are not supported and will yield `None`.
fn into_value(value: &Value, s: String) -> Option<Value> {
    use std::str::FromStr;

    match *value {
        Value::String(_)  => Some(Value::String(s)),
        Value::Integer(_) => FromStr::from_str(&s[..]).ok().map(Value::Integer),
        Value::Float(_)   => FromStr::from_str(&s[..]).ok().map(Value::Float),
        Value::Boolean(_) => {
            if s == "true" { Some(Value::Boolean(true)) }
            else if s == "false" { Some(Value::Boolean(false)) }
            else { None }
        }
        Value::Datetime(_) => Value::try_from(s).ok(),
        _ => None,
    }
}

pub trait InternalConfiguration {
    fn enable_logging(&self) -> bool {
        true
    }

    fn use_inmemory_fs(&self) -> bool {
        false
    }
}

impl<'a> InternalConfiguration for App<'a, 'a> {}

