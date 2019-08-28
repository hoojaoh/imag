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
use std::process::Command;
use std::env;
use std::process::exit;
use std::io::Stdin;
use std::io::StdoutLock;
use std::borrow::Borrow;
use std::result::Result as RResult;

pub use clap::App;
use clap::AppSettings;
use toml::Value;
use toml_query::read::TomlValueReadExt;

use clap::{Arg, ArgMatches};
use failure::ResultExt;
use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;

use crate::configuration::{fetch_config, override_config, InternalConfiguration};
use crate::logger::ImagLogger;
use crate::io::OutputProxy;

use libimagerror::exit::ExitCode;
use libimagerror::errors::ErrorMsg as EM;
use libimagerror::trace::*;
use libimagerror::io::ToExitCode;
use libimagstore::store::Store;
use libimagstore::storeid::StoreId;
use libimagutil::debug_result::DebugResult;
use crate::spec::CliSpec;
use atty;

/// The Runtime object
///
/// This object contains the complete runtime environment of the imag application running.
#[derive(Debug)]
pub struct Runtime<'a> {
    rtp: PathBuf,
    configuration: Option<Value>,
    cli_matches: ArgMatches<'a>,
    store: Store,

    has_output_pipe: bool,
    has_input_pipe: bool,

    ignore_ids: bool
}

impl<'a> Runtime<'a> {

    /// Gets the CLI spec for the program and retreives the config file path (or uses the default on
    /// in $HOME/.imag/config, $XDG_CONFIG_DIR/imag/config or from env("$IMAG_CONFIG")
    /// and builds the Runtime object with it.
    ///
    /// The cli_app object should be initially build with the ::get_default_cli_builder() function.
    pub fn new<C>(cli_app: C) -> Result<Runtime<'a>>
        where C: Clone + CliSpec<'a> + InternalConfiguration
    {
        let matches = cli_app.clone().matches();

        let rtp = get_rtp_match(&matches)?;

        let configpath = matches.value_of("config")
                                .map_or_else(|| rtp.clone(), PathBuf::from);

        debug!("Config path = {:?}", configpath);

        let config = match fetch_config(&configpath)? {
            None => {
                return Err(err_msg("No configuration file found"))
                    .context(err_msg("Maybe try to use 'imag-init' to initialize imag?"))
                    .context(err_msg("Continuing without configuration file"))
                    .context(err_msg("Cannot instantiate runtime"))
                    .map_err(Error::from);
            },
            Some(mut config) => {
                if let Err(e) = override_config(&mut config, get_override_specs(&matches)) {
                    error!("Could not apply config overrides");
                    trace_error(&e);
                }

                Some(config)
            }
        };

        Runtime::_new(cli_app, matches, config)
    }

    /// Builds the Runtime object using the given `config`.
    pub fn with_configuration<C>(cli_app: C, config: Option<Value>) -> Result<Runtime<'a>>
        where C: Clone + CliSpec<'a> + InternalConfiguration
    {
        let matches = cli_app.clone().matches();
        Runtime::_new(cli_app, matches, config)
    }

    fn _new<C>(cli_app: C, matches: ArgMatches<'a>, config: Option<Value>) -> Result<Runtime<'a>>
    where C: Clone + CliSpec<'a> + InternalConfiguration
    {
        if cli_app.enable_logging() {
            Runtime::init_logger(&matches, config.as_ref())
        }

        let rtp = get_rtp_match(&matches)?;

        let storepath = matches.value_of("storepath")
                                .map_or_else(|| {
                                    let mut spath = rtp.clone();
                                    spath.push("store");
                                    spath
                                }, PathBuf::from);

        debug!("RTP path    = {:?}", rtp);
        debug!("Store path  = {:?}", storepath);
        debug!("CLI         = {:?}", matches);
        trace!("Config      = {:#?}", config);

        let store_result = if cli_app.use_inmemory_fs() {
            Store::new_inmemory(storepath, &config)
        } else {
            Store::new(storepath, &config)
        };

        let has_output_pipe = !atty::is(atty::Stream::Stdout);
        let has_input_pipe  = !atty::is(atty::Stream::Stdin);
        let ignore_ids      = matches.is_present("ignore-ids");

        debug!("has output pipe = {}", has_output_pipe);
        debug!("has input pipe  = {}", has_input_pipe);
        debug!("ignore ids      = {}", ignore_ids);

        store_result.map(|store| Runtime {
            cli_matches: matches,
            configuration: config,
            rtp,
            store,

            has_output_pipe,
            has_input_pipe,
            ignore_ids,
        })
        .context(err_msg("Cannot instantiate runtime"))
        .map_err(Error::from)
    }

    ///
    /// Get a commandline-interface builder object from `clap`
    ///
    /// This commandline interface builder object already contains some predefined interface flags:
    ///   * -v | --verbose for verbosity
    ///   * --debug for debugging
    ///   * -c <file> | --config <file> for alternative configuration file
    ///   * -r <path> | --rtp <path> for alternative runtimepath
    ///   * --store <path> for alternative store path
    /// Each has the appropriate help text included.
    ///
    /// The `appname` shall be "imag-<command>".
    ///
    pub fn get_default_cli_builder(appname: &'a str,
                                   version: &'a str,
                                   about: &'a str)
        -> App<'a, 'a>
    {
        App::new(appname)
            .version(version)
            .author("Matthias Beyer <mail@beyermatthias.de>")
            .about(about)
            .settings(&[AppSettings::AllowExternalSubcommands])
            .arg(Arg::with_name("verbosity")
                .short("v")
                .long("verbose")
                .help("Set log level")
                .required(false)
                .takes_value(true)
                .possible_values(&["trace", "debug", "info", "warn", "error"])
                .value_name("LOGLEVEL"))

            .arg(Arg::with_name("debugging")
                .long("debug")
                .help("Enables debugging output. Shortcut for '--verbose debug'")
                .required(false)
                .takes_value(false))

            .arg(Arg::with_name("no-color-output")
                .long("no-color")
                .help("Disable color output")
                .required(false)
                .takes_value(false))

            .arg(Arg::with_name("config")
                .long("config")
                .help("Path to alternative config file")
                .required(false)
                .validator(::libimagutil::cli_validators::is_existing_path)
                .takes_value(true))

            .arg(Arg::with_name("config-override")
                 .long("override-config")
                 .help("Override a configuration settings. Use 'key=value' pairs, where the key is a path in the TOML configuration. The value must be present in the configuration and be convertible to the type of the configuration setting. If the argument does not contain a '=', it gets ignored. Setting Arrays and Tables is not yet supported.")
                 .required(false)
                 .takes_value(true))

            .arg(Arg::with_name("runtimepath")
                .long("rtp")
                .help("Alternative runtimepath")
                .required(false)
                .validator(::libimagutil::cli_validators::is_directory)
                .takes_value(true))

            .arg(Arg::with_name("storepath")
                .long("store")
                .help("Alternative storepath. Must be specified as full path, can be outside of the RTP")
                .required(false)
                .validator(::libimagutil::cli_validators::is_directory)
                .takes_value(true))

            .arg(Arg::with_name("editor")
                .long("editor")
                .help("Set editor")
                .required(false)
                .takes_value(true))

            .arg(Arg::with_name("ignore-ids")
                .long("ignore-ids")
                .help("Do not assume that the output is a pipe to another imag command. This overrides the default behaviour where imag only prints the IDs of the touched entries to stdout if stdout is a pipe.")
                .long_help("Without this flag, imag assumes that if stdout is a pipe, the command imag pipes to is also an imag command. Thus, it prints the IDs of the processed entries to stdout and automatically redirects the command output to stderr. By providing this flag, this behaviour gets overridden: The IDs are not printed at all and the normal output is printed to stdout.")
                .required(false)
                .takes_value(false))

    }

    /// Extract the Store object from the Runtime object, destroying the Runtime object
    ///
    /// # Warning
    ///
    /// This function is for testing _only_! It can be used to re-build a Runtime object with an
    /// alternative Store.
    #[cfg(feature = "testing")]
    pub fn extract_store(self) -> Store {
        self.store
    }

    /// Re-set the Store object within
    ///
    /// # Warning
    ///
    /// This function is for testing _only_! It can be used to re-build a Runtime object with an
    /// alternative Store.
    #[cfg(feature = "testing")]
    pub fn with_store(mut self, s: Store) -> Self {
        self.store = s;
        self
    }

    #[cfg(feature = "pub_logging_initialization")]
    pub fn init_logger(matches: &ArgMatches, config: Option<&Value>) {
        Self::_init_logger(matches, config)
    }
    #[cfg(not(feature = "pub_logging_initialization"))]
    fn init_logger(matches: &ArgMatches, config: Option<&Value>) {
        Self::_init_logger(matches, config)
    }

    /// Initialize the internal logger
    ///
    /// If the environment variable "IMAG_LOG_ENV" is set, this simply
    /// initializes a env-logger instance. Errors are ignored in this case.
    /// If the environment variable is not set, this initializes the internal imag logger. On
    /// error, this exits (as there is nothing we can do about that)
    fn _init_logger(matches: &ArgMatches, config: Option<&Value>) {
        use log::set_max_level;
        use log::set_boxed_logger;
        use std::env::var as env_var;

        if env_var("IMAG_LOG_ENV").is_ok() {
            let _ = env_logger::try_init();
        } else {
            let logger = ImagLogger::new(matches, config)
                .map_err_trace()
                .unwrap_or_else(|_| exit(1));

            set_max_level(logger.global_loglevel().to_level_filter());

            // safe debug output for later, after the instance itself
            // is moved away
            #[allow(unused)]
            let logger_expl = format!("{:?}", logger);

            set_boxed_logger(Box::new(logger))
                .map_err(|e| panic!("Could not setup logger: {:?}", e))
                .ok();

            trace!("Imag Logger = {}", logger_expl);
        }
    }

    /// Get the verbosity flag value
    pub fn is_verbose(&self) -> bool {
        self.cli_matches.is_present("verbosity")
    }

    /// Get the debugging flag value
    pub fn is_debugging(&self) -> bool {
        self.cli_matches.is_present("debugging")
    }

    /// Get the runtimepath
    pub fn rtp(&self) -> &PathBuf {
        &self.rtp
    }

    /// Get the commandline interface matches
    pub fn cli(&self) -> &ArgMatches {
        &self.cli_matches
    }

    pub fn ids_from_stdin(&self) -> bool {
        self.has_input_pipe
    }

    pub fn ids<T: IdPathProvider>(&self) -> Result<Option<Vec<StoreId>>> {
        use std::io::Read;

        if self.has_input_pipe {
            trace!("Getting IDs from stdin...");
            let stdin    = ::std::io::stdin();
            let mut lock = stdin.lock();

            let mut buf = String::new();
            lock.read_to_string(&mut buf)
                .context("Failed to read stdin to buffer")
                .map_err(Error::from)
                .and_then(|_| {
                    trace!("Got IDs = {}", buf);
                    buf.lines()
                        .map(PathBuf::from)
                        .map(|id| StoreId::new(id).map_err(Error::from))
                        .collect()
                })
                .map(Some)
        } else {
            Ok(T::get_ids(self.cli())?)
        }
    }

    /// Get the configuration object
    pub fn config(&self) -> Option<&Value> {
        self.configuration.as_ref()
    }

    /// Get the store object
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Get a editor command object which can be called to open the $EDITOR
    pub fn editor(&self) -> Result<Option<Command>> {
        self.cli()
            .value_of("editor")
            .map(String::from)
            .ok_or_else(|| {
                self.config()
                    .ok_or_else(|| err_msg("No Configuration!"))
                    .and_then(|v| match v.read("rt.editor")? {
                        Some(&Value::String(ref s)) => Ok(Some(s.clone())),
                        Some(_) => Err(err_msg("Type error at 'rt.editor', expected 'String'")),
                        None    => Ok(None),
                    })
            })
            .or_else(|_| env::var("EDITOR"))
            .map_err(|_| Error::from(EM::IO))
            .map_dbg(|s| format!("Editing with '{}'", s))
            .and_then(|s| {
                let mut split = s.split_whitespace();
                let command   = split.next();
                if command.is_none() {
                    return Ok(None)
                }
                let mut c = Command::new(command.unwrap()); // secured above
                c.args(split);
                c.stdin(::std::fs::File::open("/dev/tty").context(EM::IO)?);
                c.stderr(::std::process::Stdio::inherit());
                Ok(Some(c))
            })
    }

    pub fn output_is_pipe(&self) -> bool {
        self.has_output_pipe
    }

    /// Check whether the runtime ignores touched ids
    ///
    /// "Ignoring" in this context means whether the runtime prints them or not.
    pub fn ignore_ids(&self) -> bool {
        self.ignore_ids
    }

    pub fn stdout(&self) -> OutputProxy {
        if self.output_is_pipe() && !self.ignore_ids {
            OutputProxy::Err(::std::io::stderr())
        } else {
            OutputProxy::Out(::std::io::stdout())
        }
    }

    pub fn stderr(&self) -> OutputProxy {
        OutputProxy::Err(::std::io::stderr())
    }

    pub fn stdin(&self) -> Option<Stdin> {
        if self.has_input_pipe {
            None
        } else {
            Some(::std::io::stdin())
        }
    }

    /// Helper for handling subcommands which are not available.
    ///
    /// # Example
    ///
    /// For example someone calls `imag foo bar`. If `imag-foo` is in the $PATH, but it has no
    /// subcommand `bar`, the `imag-foo` binary is able to automatically forward the invokation to a
    /// `imag-foo-bar` binary which might be in $PATH.
    ///
    /// It needs to call `Runtime::handle_unknown_subcommand` with the following parameters:
    ///
    /// 1. The "command" which was issued. In the example this would be `"imag-foo"`
    /// 2. The "subcommand" which is missing: `"bar"` in the example
    /// 3. The `ArgMatches` object from the call, so that this routine can forward all flags passed
    ///    to the `bar` subcommand.
    ///
    /// # Warning
    ///
    /// If, and only if, the subcommand does not exist (as in `::std::io::ErrorKind::NotFound`),
    /// this function exits with 1 as exit status.
    ///
    /// # Return value
    ///
    /// On success, the exit status object of the `Command` invocation is returned.
    ///
    /// # Details
    ///
    /// The `IMAG_RTP` variable is set for the child process. It is set to the current runtime path.
    ///
    /// Stdin, stdout and stderr are inherited to the child process.
    ///
    /// This function **blocks** until the child returns.
    ///
    pub fn handle_unknown_subcommand<S: AsRef<str>>(&self,
                                                    command: S,
                                                    subcommand: S,
                                                    args: &ArgMatches)
        -> Result<::std::process::ExitStatus>
    {
        use std::io::Write;
        use std::io::ErrorKind;

        let rtp_str = self.rtp()
            .to_str()
            .map(String::from)
            .ok_or_else(|| Error::from(EM::IO))?;

        let command = format!("{}-{}", command.as_ref(), subcommand.as_ref());

        let subcommand_args = args.values_of("")
            .map(|sx| sx.map(String::from).collect())
            .unwrap_or_else(|| vec![]);

        Command::new(&command)
            .stdin(::std::process::Stdio::inherit())
            .stdout(::std::process::Stdio::inherit())
            .stderr(::std::process::Stdio::inherit())
            .args(&subcommand_args[..])
            .env("IMAG_RTP", rtp_str)
            .spawn()
            .and_then(|mut c| c.wait())
            .map_err(|e| match e.kind() {
                ErrorKind::NotFound => {
                    let mut out = self.stdout();

                    if let Err(e) = writeln!(out, "No such command: '{}'", command) {
                        return e;
                    }
                    if let Err(e) = writeln!(out, "See 'imag --help' for available subcommands") {
                        return e;
                    }

                    ::std::process::exit(1)
                },
                _ => e,
            })
            .context(EM::IO)
            .map_err(Error::from)
    }

    pub fn report_touched(&self, id: &StoreId) -> RResult<(), ExitCode> {
        let out      = ::std::io::stdout();
        let mut lock = out.lock();

        self.report_touched_id(id, &mut lock)
    }

    pub fn report_all_touched<ID, I>(&self, ids: I) -> RResult<(), ExitCode>
        where ID: Borrow<StoreId> + Sized,
              I: Iterator<Item = ID>
    {
        let out      = ::std::io::stdout();
        let mut lock = out.lock();

        for id in ids {
            self.report_touched_id(id.borrow(), &mut lock)?;
        }

        Ok(())
    }

    #[inline]
    fn report_touched_id(&self, id: &StoreId, output: &mut StdoutLock) -> RResult<(), ExitCode> {
        use std::io::Write;

        if self.output_is_pipe() && !self.ignore_ids {
            trace!("Reporting: {} to {:?}", id, output);
            writeln!(output, "{}", id).to_exit_code()
        } else {
            Ok(())
        }
    }
}

/// A trait for providing ids from clap argument matches
///
/// This trait can be implement on a type so that it can provide IDs when given a ArgMatches
/// object.
/// It can be used with Runtime::ids(), and libimagrt handles "stdin-provides-ids" cases
/// automatically:
///
/// ```ignore
/// runtime.ids::<PathProvider>()?.iter().for_each(|id| /* ... */)
/// ```
///
/// libimagrt does not call the PathProvider if the ids are provided by piping to stdin.
///
///
/// # Passed arguments
///
/// The arguments which are passed into the IdPathProvider::get_ids() function are the _top level
/// ArgMatches_. Traversing might be required in the implementation of the ::get_ids() function.
///
///
/// # Returns
///
/// In case no store ids could be matched, the function should return `Ok(None)` instance.
/// On success, the StoreId objects to operate on are returned from the ArgMatches.
/// Otherwise, an appropriate error may be returned.
///
pub trait IdPathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>>;
}

/// Exported for the `imag` command, you probably do not want to use that.
pub fn get_rtp_match<'a>(matches: &ArgMatches<'a>) -> Result<PathBuf> {
    if let Some(p) = matches
        .value_of("runtimepath")
        .map(PathBuf::from)
    {
        return Ok(p)
    }

    match env::var("IMAG_RTP").map(PathBuf::from) {
        Ok(p) => return Ok(p),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(err_msg("Environment variable 'IMAG_RTP' does not contain valid Unicode"))
        },
        Err(env::VarError::NotPresent) => { /* passthrough */ }
    }

    env::var("HOME")
        .map(PathBuf::from)
        .map(|mut p| { p.push(".imag"); p })
        .map_err(|e| match e {
            env::VarError::NotUnicode(_) => {
                err_msg("Environment variable 'HOME' does not contain valid Unicode")
            },
            env::VarError::NotPresent => {
                err_msg("You seem to be $HOME-less. Please get a $HOME before using this \
                    software. We are sorry for you and hope you have some \
                    accommodation anyways.")
            }
        })
}

fn get_override_specs(matches: &ArgMatches) -> Vec<String> {
    matches
        .values_of("config-override")
        .map(|values| {
             values
             .filter(|s| {
                 let b = s.contains('=');
                 if !b { warn!("override '{}' does not contain '=' - will be ignored!", s); }
                 b
             })
             .map(String::from)
             .collect()
        })
        .unwrap_or_else(|| vec![])
}

