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

use std::io::Write;
use std::io::stderr;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::ops::Deref;

use crate::runtime::Runtime;

use failure::ResultExt;
use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;
use clap::ArgMatches;
use log::{Log, Level, Record, Metadata};
use toml::Value;
use toml_query::read::TomlValueReadExt;
use toml_query::read::TomlValueReadTypeExt;
use handlebars::Handlebars;

use libimagerror::errors::ErrorMsg as EM;

type ModuleName = String;

#[derive(Debug)]
enum LogDestination {
    Stderr,
    File(Arc<Mutex<::std::fs::File>>),
}

impl Default for LogDestination {
    fn default() -> LogDestination {
        LogDestination::Stderr
    }
}

#[derive(Debug)]
struct ModuleSettings {
    enabled:        bool,
    level:          Option<Level>,

    #[allow(unused)]
    destinations:   Option<Vec<LogDestination>>,
}

/// Logger implementation for `log` crate.
#[derive(Debug)]
pub struct ImagLogger {
    global_loglevel     : Level,

    #[allow(unused)]
    global_destinations : Vec<LogDestination>,
    // global_format_trace : ,
    // global_format_debug : ,
    // global_format_info  : ,
    // global_format_warn  : ,
    // global_format_error : ,
    module_settings     : BTreeMap<ModuleName, ModuleSettings>,

    handlebars: Handlebars,
}

impl ImagLogger {

    /// Create a new ImagLogger object with a certain level
    pub fn new(matches: &ArgMatches, config: Option<&Value>) -> Result<ImagLogger> {
        let mut handlebars = Handlebars::new();

        handlebars.register_escape_fn(::handlebars::no_escape);

        ::libimaginteraction::format::register_all_color_helpers(&mut handlebars);
        ::libimaginteraction::format::register_all_format_helpers(&mut handlebars);

        {
            use self::log_lvl_aggregate::*;
            let _ = aggregate::<Trace>(&mut handlebars, config, "TRACE")?;
            let _ = aggregate::<Debug>(&mut handlebars, config, "DEBUG")?;
            let _ = aggregate::<Info>(&mut handlebars, config, "INFO")?;
            let _ = aggregate::<Warn>(&mut handlebars, config, "WARN")?;
            let _ = aggregate::<Error>(&mut handlebars, config, "ERROR")?;
        }

        Ok(ImagLogger {
            global_loglevel     : aggregate_global_loglevel(matches, config)?,
            global_destinations : aggregate_global_destinations(config)?,
            module_settings     : aggregate_module_settings(matches, config)?,
            handlebars          : handlebars,
        })
    }

    pub fn global_loglevel(&self) -> Level {
        self.global_loglevel
    }

}

impl Log for ImagLogger {

    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.global_loglevel
    }

    fn flush(&self) {
        // nothing?
    }

    fn log(&self, record: &Record) {
        let mut data = BTreeMap::new();

        {
            data.insert("level",        format!("{}", record.level()));
            data.insert("module_path",  String::from(record.module_path().unwrap_or("<modulepath unknown>")));
            data.insert("file",         String::from(record.file().unwrap_or("<file unknown>")));
            data.insert("line",         format!("{}", record.line().unwrap_or(0)));
            data.insert("target",       String::from(record.target()));
            data.insert("message",      format!("{}", record.args()));
        }

        let logtext = self
            .handlebars
            .render(&format!("{}", record.level()), &data)
            .unwrap_or_else(|e| format!("Failed rendering logging data: {:?}\n", e));

        let log_to_destination = |d: &LogDestination| match d {
            &LogDestination::Stderr => {
                let _ = write!(stderr(), "{}\n", logtext);
            },
            &LogDestination::File(ref arc_mutex_logdest) => {
                // if there is an error in the lock, we cannot do anything. So we ignore it here.
                let _ = arc_mutex_logdest
                    .deref()
                    .lock()
                    .map(|mut logdest| {
                        write!(logdest, "{}\n", logtext)
                    });
            }
        };

        // hack to get the right target configuration.
        // If there is no element here, we use the empty string which automatically drops through
        // to the unwrap_or_else() case
        let record_target = record
            .target()
            .split("::")
            .next()
            .unwrap_or("");

        self.module_settings
            .get(record_target)
            .map(|module_setting| {
                let set = module_setting.enabled &&
                    module_setting.level.unwrap_or(self.global_loglevel) >= record.level();

                if set {
                    module_setting.destinations.as_ref().map(|destinations| for d in destinations {
                        // If there's an error, we cannot do anything, can we?
                        let _ = log_to_destination(&d);
                    });

                    for d in self.global_destinations.iter() {
                        // If there's an error, we cannot do anything, can we?
                        let _ = log_to_destination(&d);
                    }
                }
            })
        .unwrap_or_else(|| {
            if self.global_loglevel >= record.level() {
                // Yes, we log
                for d in self.global_destinations.iter() {
                    // If there's an error, we cannot do anything, can we?
                    let _ = log_to_destination(&d);
                }
            }
        });
    }
}

fn match_log_level_str(s: &str) -> Result<Level> {
    match s {
        "trace" => Ok(Level::Trace),
        "debug" => Ok(Level::Debug),
        "info"  => Ok(Level::Info),
        "warn"  => Ok(Level::Warn),
        "error" => Ok(Level::Error),
        lvl     => return Err(format_err!("Invalid logging level: {}", lvl)),
    }
}

fn aggregate_global_loglevel(matches: &ArgMatches, config: Option<&Value>) -> Result<Level>
{
    fn get_arg_loglevel(matches: &ArgMatches) -> Result<Option<Level>> {
        if matches.is_present(Runtime::arg_debugging_name()) {
            return Ok(Some(Level::Debug))
        }

        match matches.value_of(Runtime::arg_verbosity_name()) {
            Some(v) => match_log_level_str(v).map(Some),
            None    => if matches.is_present(Runtime::arg_verbosity_name()) {
                Ok(Some(Level::Info))
            } else {
                Ok(None)
            },
        }
    }

    if let Some(cfg) = config {
        let cfg_loglevel = cfg
            .read_string("imag.logging.level")
            .map_err(Error::from)
            .context(EM::TomlQueryError)?
            .ok_or(err_msg("Global log level config missing"))
            .and_then(|s| match_log_level_str(&s))?;

        if let Some(cli_loglevel) = get_arg_loglevel(matches)? {
            if cli_loglevel > cfg_loglevel {
                return Ok(cli_loglevel)
            }
        }

        Ok(cfg_loglevel)

    } else {
        get_arg_loglevel(matches).map(|o| o.unwrap_or(Level::Info))
    }
}

fn translate_destination(raw: &str) -> Result<LogDestination> {
    use std::fs::OpenOptions;

    match raw {
        "-" => Ok(LogDestination::Stderr),
        other => {
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(other)
                .map(Mutex::new)
                .map(Arc::new)
                .map(LogDestination::File)
                .map_err(Error::from)
                .context(EM::IO)
                .map_err(Error::from)
        }
    }
}


fn translate_destinations(raw: &Vec<Value>) -> Result<Vec<LogDestination>> {
    raw.iter()
        .map(|val| {
            val.as_str()
                .ok_or_else(|| "Type error at 'imag.logging.modules.<mod>.destinations', expected Array<String>")
                .map_err(err_msg)
                .map_err(Error::from)
                .and_then(|s| translate_destination(s))
        })
        .collect()
}

fn aggregate_global_destinations(config: Option<&Value>)
    -> Result<Vec<LogDestination>>
{
    match config {
        None      => Ok(vec![LogDestination::default()]),
        Some(cfg) => cfg
            .read("imag.logging.destinations")
            .map_err(Error::from)
            .context(EM::TomlQueryError)?
            .ok_or_else(|| err_msg("Global log destination config missing"))?
            .as_array()
            .ok_or_else(|| {
                let msg = "Type error at 'imag.logging.destinations', expected 'Array'";
                Error::from(err_msg(msg))
            })
            .and_then(translate_destinations),
    }
}

mod log_lvl_aggregate {
    use failure::Fallible as Result;
    use failure::Error as E;
    use failure::ResultExt;
    use failure::err_msg;
    use toml::Value;
    use toml_query::read::TomlValueReadTypeExt;
    use handlebars::Handlebars;

    use libimagerror::errors::ErrorMsg as EM;

    macro_rules! aggregate_global_format_with {
        ($t:ident, $read_str:expr) => {
            pub struct $t;
            impl LogLevelAggregator for $t {
                fn aggregate(config: Option<&Value>) -> Result<String> {
                    config.ok_or_else(|| {
                        E::from(err_msg(concat!("Config missing: Logging format: ", stringify!($t))))
                    })?
                    .read_string($read_str)
                    .map_err(E::from)
                    .context(EM::TomlQueryError)?
                    .ok_or_else(|| {
                        E::from(err_msg(concat!("Config missing: Logging format: ", stringify!($t))))
                    })
                }
            }
        };
    }

    pub trait LogLevelAggregator {
        fn aggregate(config: Option<&Value>) -> Result<String>;
    }

    pub fn aggregate<T: LogLevelAggregator>(hb: &mut Handlebars, config: Option<&Value>, lvlstr: &str)
        -> Result<()>
    {
        hb.register_template_string(lvlstr, T::aggregate(config)?)
            .map_err(E::from)
            .context(err_msg("Handlebars template error"))
            .map_err(E::from)
    }

    aggregate_global_format_with!(Trace, "imag.logging.format.trace");
    aggregate_global_format_with!(Debug, "imag.logging.format.debug");
    aggregate_global_format_with!(Info, "imag.logging.format.info");
    aggregate_global_format_with!(Warn, "imag.logging.format.warn");
    aggregate_global_format_with!(Error, "imag.logging.format.error");

}

fn aggregate_module_settings(_matches: &ArgMatches, config: Option<&Value>)
    -> Result<BTreeMap<ModuleName, ModuleSettings>>
{
    use toml_query::read::Partial;
    use std::convert::TryInto;

    #[derive(Serialize, Deserialize, Debug)]
    struct LoggingModuleConfig {
        pub destinations: Option<Vec<String>>,
        pub level: Option<Level>,
        pub enabled: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct LoggingModuleConfigMap(BTreeMap<String, LoggingModuleConfig>);

    impl<'a> Partial<'a> for LoggingModuleConfigMap {
        const LOCATION: &'static str = "imag.logging.modules";
        type Output                  = Self;
    }

    impl TryInto<BTreeMap<String, ModuleSettings>> for LoggingModuleConfigMap {
        type Error = Error;

        fn try_into(self) -> Result<BTreeMap<String, ModuleSettings>> {
            let mut map = BTreeMap::new();

            for (key, value) in self.0.into_iter() {
                map.insert(key, ModuleSettings {
                    enabled:      value.enabled,
                    level:        value.level.map(Into::into),
                    destinations: match value.destinations {
                        None     => None,
                        Some(ds) => Some(ds
                            .iter()
                            .map(Deref::deref)
                            .map(translate_destination)
                            .collect::<Result<Vec<LogDestination>>>()?)
                    },
                });
            }

            Ok(map)
        }
    }

    match config {
        Some(cfg) => cfg.read_partial::<LoggingModuleConfigMap>()?
            .ok_or_else(|| err_msg("Logging configuration missing"))?
            .try_into()
            .map_err(Error::from),
        None => {
            write!(stderr(), "No Configuration.").ok();
            write!(stderr(), "cannot find module-settings for logging.").ok();
            write!(stderr(), "Will use global defaults").ok();

            Ok(BTreeMap::new())
        }
    }
}

