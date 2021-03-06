# Changelog {#sec:changelog}

This section contains the changelog.

We try to include a changelog line in each pull request, to make sure the
changelog is up to date when releasing a new version of the codebase.
Make sure to append the new change to the list, do not prepend it.

The "Major" section of each section includes huge changes in functionality and
interfaces (but not necessarily user-facing ones), whereas the "Minor" section
contains only small stuff.
Some things, like typo fixes, version string updates and such are not stated in
the changelog (though updating of dependencies is).
Please note that we do not have a "Breaking changes" section as we are in
Version 0.y.z and thus we can break the API like we want and need to.

## 0.9.3

Bugfix release for fixing:

* Removed an import which was already there and fails with the current beta
  compiler
* Fix a negation error in the config aggregation in imag-log
* Dependency specification fail in 0.9.2 where everything did not depend on
  0.9.2 but 0.9.1

## 0.9.2

Bugfix release for fixing:

* Fix a function that checks a flag. If the flag is not there, it should be "not
  set".
* Fix to not ignore errors when collecting links in libimagentrylink
* Remove buildscripts because imag was not installable from crates.io with
  buildscripts.


## 0.9.1

Bugfix release for fixing:

* Fix off by one error in error tracing
* Fix negation error in imag-habit filtering mechanism
* Fix config override mechanism
* "status" subcommand might not be present in imag-habit, but standard command
  should work anyways
* We go to the next month, not to the current one (off by one error)
* 'start-time' cannot be None in imag-timetrack, clap ensures that
* Do not use deprecated StoreId::exists() function in libimagentrycategory


## 0.9.0

* [f912d3e7f3](https://git.imag-pim.org/imag/commit/?id=f912d3e7f362e524347cd061f316d3569dfb18a0)
  Fix: Duplicated printing of output in `imag log show --all`
  The problem was that the used `Diary::diary_names()` iterator does not
  call `unique()` on its output.
  That decision was made because the return type would get more
  complicated with that feature.
  The fix was to call `Itertools::unique()` on the iterator.

* [851db4abe4](https://git.imag-pim.org/imag/commit/?id=851db4abe4d8d0aebafe35ef41c5594213b1be6b)
  Do not use rust-crypto anymore, but other crates
  A contribution from newpavlov. Thank you very much!

* [5b82d53fd2](https://git.imag-pim.org/imag/commit/?id=5b82d53fd29294ac598d3e80fcee0e7b27f59c0c)
  Optimize libimagstore filesystem backend
  This optimization changes the backend so that the files are not held open. The
  files are now read whenever they are requested, then they are cached and are
  written back on "Store::update()" (which is also called when the store is
  dropped).
  This change allows us now to read more files into memory than there are FDs for
  the process (which was a problem with really large stores and, for example,
  imag-diagnostics.

* [6a81c0afd1](https://git.imag-pim.org/imag/commit/?id=6a81c0afd1d98520d4820f6a1dcb161ee4029761)
  Update rust compiler for travis

* [ccbc2b2672](https://git.imag-pim.org/imag/commit/?id=ccbc2b2672defc61f4e67040fc193b8271bd9f60)
  Add progress bar for imag-diagnostic tool, so that a user can see
  that something is happening.

* [a101e777f3](https://git.imag-pim.org/imag/commit/?id=a101e777f365bc1e1fcd9f4ca3f9a9b6198e70af)
  Update dependencies
  Dependencies of these imag crates where updated:
  Binaries:
  * imag
  * imag-contact
  * imag-edit
  * imag-git
  * imag-grep
  * imag-habit
  * imag-link
  * imag-timetrack
  * imag-view
  * imag-wiki

  Libraries:
  * libimagbookmark
  * libimagcontact
  * libimagentryfilter
  * libimagentrytag
  * libimagentryview
  * libimagerror
  * libimaginteraction
  * libimagmail
  * libimagrt
  * libimagstore
  * libimagtimeui
  * libimagtodo
  * libimagutil
  * libimagwiki

* [9bce68b1bf](https://git.imag-pim.org/imag/commit/?id=9bce68b1bff5e2f77f7c86424de5c16a7e38a2cc)
  Optimized the libimagstore `Store::entries()` interface to have
  the possibility to limit the filesystem access to subdirectories in the store
  path, so that IO operations are minimized.

* [a749d97a16](https://git.imag-pim.org/imag/commit/?id=a749d97a16af553e4a6f949538433067a757d9e8)
  Switched the whole ecosystem to use `failure` for error handling.

* [ecf4cead93](https://git.imag-pim.org/imag/commit/?id=ecf4cead93da0b04cb74370082dbfc998d7a2752)
  Introduced the runtime IO system, which can now be used to chain
  imag calls like so: `imag ids | imag tag add "foobar"`.

In the process of these major changes, small bugfixes and improvements were
applied to the codebase. Too much to list all of them here, though.

The merge messages of the respective feature branches contain more details on
the changes.


## 0.8.0

After the last release (0.7.0), we changed how we keep our changelog from manual
adding things to the list, to using `git-notes`. Hence, there's no
categorization anymore.

* Add imag(https://git.imag-pim.org/imag/commit/?id=imag)diary functionality to list existing diaries

* `libimagentryview`s `StdoutViewer` is now able to wrap lines

* `imag-view` can wrap output now

* `imag tag` is now able to read store ids from stdin

* `libimagrt` automatically suggests "imag init" if config is not found

* A changelog-generation script was added to the scripts/ directory

* Fix: `libimagdiary` did not return the youngest, but the oldest entry id on
  `::get_youngest_entry_id()`.

* Fix: imag-view should not always wrap the text, only if -w is passed

* Fix: `imag-log show` should order by date

* `imag` does not inherit stdout if detecting imag command versions.

* The `Store::retrieve_for_module()` function was removed.

* `imag-git` was added, a convenient way to call `git` in the imag RTP.

* `libimagentryview` was refactored.
  The crate was refactored so that the "sink" - the thing the view should be
  written to - can be passed.

* A `imag-view` feature was added where markdown output can be formatted nicely
  for the terminal

* The libimagstore lib got its stdio backend removed.

  First, `imag store dump` was removed as it was based on this feature.
  Then, `libimagrt` got the ability removed to change the store backend to stdio.
  After that, we were able to remove the stdio backend and the JSON mapper
  functionality that came with it.

  This shrinked the codebase about 1kloc.

  The `imag store dump` feature can be reimplemented rather simply, if desired.

* `imag-view` is now able to seperate printed entries with a user-defined
  character (default: "-")

* Fix: Deny non-absolute import pathes in imag-contact

* libimagcontact is not based on libimagentryref anymore

  This is because we encountered a serious problem: When syncing contacts with an
  external tool (for example `vdirsyncer`), the files are renamed on the other
  host. Syncing the imag store to the other device now creates dead links, as the
  `path` stored by the ref is not valid anymore.

  Now that libimagcontact is not based on libimagentryref anymore, this issue does
  not exist anymore. libimagcontact stores all contact information inside the
  store now.

  imag-contact was rewritten for that change.

* Fix: imag-contact does only require the name field, all others are optional
  now

* Fix: imag-contact automatically creates a UID now

* `libimagcategory` was rewritten

  It creates entries for categories now and automatically links categorized
  entries to the "category" entries.

  Its codebase got a bit simpler with these changes, despite the increase of
  functionality.

* `imag-contact` automatically generates/warns about missing file extension

* `libimagcontact` does export email properties now.

  `imag-contact` reads email properties and can output them in its JSON output.
  This is helpful for passing email adresses to external tools like mutt.

* `libimagentrygps` and `imag-gps` work with 64 bit signed values now

  Both the library and the command use i64 (64 bit signed) for GPS value fragments
  now.

  Also: The `imag-gps` tool does not require a "second" value fragment now, it
  defaults to 0 (zero) if not present.

* The `filters` dependency was updated to 0.3

* `libimagentryfilter` filters headers not with a failable filter.

* `imag-diary` has no longer an `edit` command. `imag-edit` shall be used
  instead.

* `libimagtodo` got a error handling refactoring, so that more chaining happens.

* Errors in `libimagstore` contain more details in the error message about what
  StoreId caused the error
  Unused errors were removed.

* The Store API got functions to get data about the internal cache and flush it

* imag-diagnostics flushes the cache each 100 entries processed

* The iterator interface of libimagstore was refactored

  Originally, the iterator interface of libimagstore did not return errors which
  could happen during the construction of a `StoreId` object.
  This was refactored, effectively changing the `StoreIdIterator` type to not
  iterate over `StoreId` anymore, but over `Result<StoreId, StoreError>`.
  That cause a lot of changes in the overall iterator interface. All iterator
  extensions (like `.into_get_iter()` for example) had to be rewritten to be
  applicable on iterators over `Result<StoreId, E>` where `E` is a generic that
  can be constructed `From<StoreError>`.

  This all was triggered by the fact that `Store::entries()` did a
  `.collect::<Vec<_>>()` call, which is time consuming.
  Consider a tool which calls `Store::entries()` and then only use the first 10
  entries returned (for some user-facing output for example). This would
  effectively cause the complexe store to be accessed (on the filesystem level),
  which could be thousands of entries. All relevant pathes would have been written
  to memory and then be filtered for the first 10.
  Not very optimized.

  After this was done, the store interface changed. This caused a lot of changes
  in almost all crates.

  Internally, in the store, the `FileAbstraction` object is not passed around in a
  `Box` anymore, but in an `Arc`, as a intermediate (store-private) iterator type
  needs to access the backend for file-system level checks.
  This is not user-facing.

  In the process, the `Store::reset_backend()` interface was removed (as this is
  no longer supported and should've been removed already). Rewriting it just for
  the sake of this patch and then removing it would've been to difficult and time-
  consuming, hence it was simply removed during that patchset.

  The overall performance was somewhat improved by this patchset.
  A rather non-scientifically performed test shows increased performance in debug
  builds (but slowing down in release builds).

  The test was done on master before the merge and after the merge, with a debug
  build and a release build. Each time, `imag-ids` was executed 10 times with its
  output piped to /dev/null. The local store used for this contained 5743 entries
  during the measurements and was not changed in between. `time` showed the
  following data (real, user, sys):

  Before, Debug build:

      0,075;0,052;0,023
      0,077;0,051;0,026
      0,083;0,063;0,020
      0,079;0,054;0,025
      0,076;0,057;0,019
      0,077;0,059;0,017
      0,074;0,052;0,022
      0,077;0,045;0,032
      0,080;0,060;0,020
      0,080;0,058;0,022

  After, Debug build:

      0,071;0,050;0,021
      0,073;0,053;0,021
      0,075;0,060;0,015
      0,076;0,047;0,029
      0,072;0,055;0,018
      0,077;0,061;0,016
      0,071;0,053;0,019
      0,070;0,053;0,016
      0,074;0,050;0,025
      0,076;0,052;0,024

  Before, Release build:

      0,034;0,015;0,019
      0,034;0,017;0,017
      0,034;0,019;0,015
      0,033;0,012;0,022
      0,034;0,011;0,023
      0,034;0,015;0,019
      0,034;0,010;0,024
      0,033;0,015;0,018
      0,037;0,017;0,020
      0,033;0,013;0,021

  After, Release build:

      0,037;0,015;0,022
      0,036;0,016;0,020
      0,036;0,018;0,018
      0,036;0,014;0,022
      0,036;0,015;0,021
      0,036;0,018;0,018
      0,037;0,016;0,020
      0,036;0,018;0,018
      0,039;0,015;0,023
      0,037;0,016;0,021

* The `Store::walk()` function was removed.

* `imag-ids` got the ability to filter by header

  The language introduced here is subject to change, but a good first step into
  the direction of a generic filter language.

  Language documentation was added as well and is printed with `imag ids --help`.

* `imag-category` was added

* The standard CLI interface in `libimagrt` was updated and validations were
  added


## 0.7.1

Bugfix release for fixing:

* `libimagdiary` did not return the youngest, but the oldest entry id on `::get_youngest_entry_id()`.
* `imag-view` should not always wrap the text, only if -w is passed
* `imag-log show` should order by date
* `imag` does not inherit stdout if detecting imag command versions.
* `imag-contact import` does only allow absolute pathes
* `imag-contact` has most fields optional now, only name is required
* `imag-contact` automatically creates UID
* `imag-contact` automatically generates/warns about missing file extension


## 0.7.0

* Major changes
    * `imag-timetrack list --from/--to` now have `kairos` support - that means
      that complex `--from/--to` arguments (like `yesterday` or `today-2weeks`)
      are now possible
    * `libimagerror` got a major refactoring and uses `ChainedError` from
      `error-chain` for logging errors now.
    * `libimagentryref` and all libraries using it were rewritten.
      `libimagentryref` was rewritten to make its API simpler and yet more powerful.
      Also because it used to put entries under a "ref" collection in the store,
      but users of the library really should be be able to put entries under
      custom collections.
    * `imag store ids` was replaced by `imag ids`.
    * `libimagentrylist` was removed. Its functionality was inconvenient to use
      and ugly to implement. Its API was cumbersome.
      Listing of entries shall be implemented without it.
    * `libimagcontact` is now able to fetch all contacts from the store.
    * `libimagcontact` takes the hash from the vcard object (UID) now.
    * `imag-contact` got a `find` command, which matches in fullname, email and
      address and either shows or lists the found contacts
    * `imag-contact list` and `imag-contact find` is now able to print the
      output as JSON.
    * `imag-edit` can now read store ids from stdin, so
      `imag ids | fzf | imag edit -I` is now a thing.
    * `imag ids` does not print the path of the store. Can be turned on using
      commandline flag.
    * `imag-habit today --done` and `imag-habit status --done` was added for
      showing habits which are already done.
    * `libimagrt` allows external subcommands now in the default clap app
      builder helper. It also provides a helper for handling unknown
      subcommands: `Runtime::handle_unknown_subcommand()`. See docs for details.
    * `imag-link list` prints output in ascii-table now, use `--plain` to print
      as plain text.
    * The build script automatically generates autocomplete scripts for bash,
      fish and zsh now when compiling the `imag` command.
    * `libimagwiki` and `imag-wiki` were introduced.
* Minor changes
    * A license-checker was included into the CI setup, which checks whether all
      ".rs"-files have the license header at the top of the file
    * `imag-link` does not allow linking the entry to itself
    * `imag` sorts available commands alphabetically now
    * `imag` has a new subcommand `help` for consistency with other tools
    * `imag-grep` does not print grep statistics when only files with matches
      are listed
    * The `"Ok"` output which was printed on success was removed from all
      commands
    * `imag-log show` was aliased to `imag-log list`
    * `imag-* --version` shows `git describe` output if binary was compiled in
      "debug" mode.
    * `imag-diary` supports "daily" diaries now.
    * `imag-contact` joins multiple emails with "," now
    * `imag-tag` commandline was rewritten for positional arguments.
    * `libimagrt` automatically takes "rt.editor" into account when building
      editor object
    * `libimagentryref` got a utility function for making an entry a ref.
    * `libimaghabit` got `Habit::instance_exists_for_date()`
    * `imag contact find` understands `--format` now.
    * `imag contact` uses ", " as seperator for output of lists of values.
    * `imag contact find --id / --full-id` was added for printing Store Id /
      Filepath of found contacts.
    * `imag view` can now view multiple entries at once
    * `imag view -I` reads store ids from stdin now.
    * `libimagstore` iterators have less restricting lifetimes now
    * `libimagentrygrep` was introduced, a crate for seaching in the
      header/content parts of an entry.
    * `imag-ids` can now filter by collection
    * All crates use "clap" with the "wrap_help" feature now.
* Bugfixes
    * imag does not panic anymore when piping and breaking that pipe, for
      example like with `imag store ids | head -n 1`.
      For that, `libimagerror` got a `Result` extension which can translate
      errors into exit codes and one for unwrapping or exiting with the
      `Err(i32)` from the result.
    * `libimagdiary` did not add the header markers on diary entries.
    * `imag-diary` used the default diary rather than the CLI setting. Now it
      rather uses the CLI setting and only if that is not present, it uses the
      default.
    * `libimagerror` printed errors with `write!()` rather than `writeln!()`
      when tracing.
    * A parsing error in `libimagstore`, which caused the parsing of entries
      with a line "---" in the content part to fail, was fixed.
    * The patch explained by the point above introduced a bug which caused
      entries to be read as a single line, which was fixed as well.
    * `imag-diary create --timed` did not work as expected
    * `libimagstore` got another fix with the file parsing, as the
      `std::str::Lines` iterator takes empty lines as no lines.
    * `libimagentryedit` fixed to inherit stdin and stderr to child process for
      editor command.
    * `libimagrt` produced the editor command without taking arguments into
      account.
    * `libimagentryref` got a fix where the buffer for the hash calculation was
      not allocated properly.
    * `libimagstore::store::Store::create` overwrote existing entries.
    * `libimaghabit::habit::HabitTemplate` did not link new instances.
    * `imag-init` creates `~/.imag` but not `~/.imag/store`.
    * `libimagrt` got a bugfix in the editor command setup where command
      arguments were not processed correctly which could result in calling the
      editor with an empty argument (`vim " "`).
    * `imag-grep` did not count in all cases.
    * `libimagdiary` sorts entries by date when viewing/listing them.
    * A `libimagentryref` bug was fixed where the wrong variable was passed as
      path to the referenced file, causing all tools based on this lib to break.
    * `libimagrt` had a bug where the logging level was set to "Info" as soon as
      "--verbose" was passed, but the value of "--verbose" was not even checked.


## 0.6.4

Bugfix release for fixing:

* `libimagrt` produced the editor command without taking arguments into
  account.
* `imag-init` creates `~/.imag` but not `~/.imag/store`.
* Fix editor setup in `libimagrt` to use /dev/tty as stdin for editor, so
  terminal-editors do not trash the terminal


## 0.6.3

Bugfix release for fixing:

* `libimagstore` got another fix with the file parsing, as the
  `std::str::Lines` iterator takes empty lines as no lines.


## 0.6.2

Bugfix release for fixing:

* `imag-diary` did not recognize the "-d DIARY" setting.
* A parsing error in `libimagstore`, which caused the parsing of entries
  with a line "---" in the content part to fail, was fixed.
* The bugfix above introduced another bug which caused entries to be rewritten
  in one line when accessing them. This was fixed.
* `imag-diary` did not properly set "minute" and "second" when creating "hourly"
  or "minutely" entries.
* Version numbers for all crates as well as in the docs were updated to "0.6.2".


## 0.6.1

Bugfix release for fixing two severe bugs in `imag-init`:

* `imag-init` created the git directory inside the imag directory. Fixed by
  defaulting to `{imag directory}/.git`.
* `imag-init` was buggy as it did not include the `imagrc.toml` file in the
  release, thus building it from crates.io failed


## 0.6.0

* Major changes
    * The config infrastructure of `libimagstore` was removed, as it was unused.
    * The iterators of `libimagstore` were improved and are now abstract over
      all iterator types. For example, all iterators over `StoreId` can now be
      transformed into a `StoreGetIterator`.
    * `imag-log` was introduced
    * `imag-init` was introduced
    * `libimagdiary` supports second-granularity now.
    * `libimagstore::store::Store::retrieve_copy` was renamed to
      `libimagstore::store::Store::get_copy`, which describes the semantics of
      the function way better.
    * `libimagentryutil` was introduced, a library for helpers for
      `libimagstore::store::Entry` handling and writing extension-writing.
    * `imag-edit` was introduced
    * `imag-diary` got second-granularity support in the CLI.
* Minor changes
    * Internals were refactored from `match`ing all the things into function
      chaining
    * The `toml-query` dependency was updated to 0.5.0
    * `imag-timetrack list` lists with a table now
    * `imag-timetrack stop` now stops all running tags if none are specified
    * The `toml-query` dependency was updated to 0.6.0
    * `ResultExt::map_err_trace_exit()` was removed in favour of
      `ResultExt::map_err_trace_exit_unwrap()`.
    * `imag-view` shows content by default now. Use `-C` to hide the content.
    * `kairos` dependency was updated to 0.1.0
* Bugfixes
    * `libimagbookmark` contained a type which wrapped a `FileLockEntry` from
      `libimagstore`. This was considered a bug and was fixed.
    * We depended on a crate which was licensed as GPLv2, which would yield imag
      GPL as well. The dependency was removed.
    * The `imag` crate prints the "command filed" error message to stderr now.
      It also prefixes the subcommand with `imag-<command>` for better user
      experience.
    * `libimagnotes` did not set the note name in the header of the note.
    * `imag-mv` automatically fixes links when moving an entry in the store.
    * `imag-log` listed non-log entries (normal diary entries) before, was
      changed to only list `log` entries.

## 0.5.0

* Major changes
    * `imag-counter` and `libimagcounter` was removed.
    * `imag-mv` was introduced
    * `imag-view` uses positional args now
    * `imag-view` uses the configuration file now to find the command to call
      for viewing the entry. This way one can view the entry in an editor or the
      browser or on the toaster.
    * The logger is now able to handle multiple destinations (file and "-" for
      stderr)
    * `imag-store` can dump all storeids now
    * `imag-annotate` was introduced
    * `imag-diagnostics` was added
    * The runtime does not read the config file for editor settings anymore.
      Specifying an editor either via CLI or via the `$EDITOR` environment
      variable still possible.
    * `imag-contact` was added (with basic contact support so far).
    * `imag-habit` was introduced
    * `imag-link` commandline was redesigned to be easier but with the same
      features.

* Minor changes
    * `libimagentryannotation` got a rewrite, is not based on `libimagnotes`
      anymore. This is minor because `libimagentryanntation` is not yet used by
      any other crate.
    * imag now reads the `IMAG_RTP` environment variable before trying to access
      `$HOME/.imag` for its runtimepath.
    * `libimagnotification` was introduced, though not yet integrated into the
      CLI tools

* Bugfixes
    * `Store::entries()` does not yield StoreIds which point to directories
      anymore, only StoreIds pointing to files.

* Stats
    * 227 commits
    * 51 merge-commits / 176 non-merge commits
    * 2 contributors
    * 186 files changed
    * 6707 insertions(+) / 3255 deletions(-)

## 0.4.0

* Major changes
    * The `libimagstore::toml_ext` module was removed. The `toml_query` crate
      should be used as a replacement. Its interface only differs in few places
      from the old `libimagstore::toml_ext` interface.
    * The codebase was moved to a more tree-ish approach, where several
      subdirectories were introduced for different types of crates
    * The documentation got a major overhaul and was partly rewritten
    * The logger is now configurable via the config file.
    * The error handling of the whole codebase is based on the `error_chain`
      now. `libimagerror` only contains convenience functionality, no
      error-generating macros or such things anymore.
    * `imag-diary` can now use a configuration in the imagrc.toml file where for
      each diary there is a config whether entries should be created minutely or
      hourly (or daily, which is when specifying nothing).
* New
    * `libimagentrygps` was introduced
    * `imag-gps` was introduced
    * `imag-grep` was introduced
    * The `imag` command now passes all arguments properly to the called
      subcommand
* Fixed bugs
    * The config loading in `libimagrt`
    [was fixed](http://git.imag-pim.org/imag/commit/?id=9193d50f96bce099665d2eb716bcaa29a8d9b8ff).
    * `libimagentrylink` used `imag` as the location for putting links in
      entries. This is not allowed because this namespace is reserved for the
      store itself. This bug was fixed, links are now located in the `links`
      namespace in the header of an entry.
    * `Store::delete()` did only check the store-internal cache whether an entry
      exists, but not the filesystem. This was fixed.
* Minor changes
    * If building from a `nix-shell`, the mozilla rust overlay is expected to be
      present
    * Unused imports in the codebase were removed
    * Compiler Warnings were fixed
    * We specify inter-dependencies via path and variable now, so one can build
      the 0.3.0 release from the checkout of the codebase.
    * The `imag` binary was refactored and rewritten, the `crossbeam`
      dependency was removed.
    * The `Makefile` was removed as `cargo` is powerful enough to fit our needs
    * `libimagstore::storeid::StoreId::is_in_collection()` was added
    * The `libimagentrylink` is now rudimentarily tested
    * We compile with rustc 1.17, 1.18, .., nightly
    * The `imag-store` binary now uses positional arguments in its CLI
    * The "toml-query" dependency was updated to 0.3.1
    * `imag-timetrack track` is now able to parse "now", date-only start/stop
      dates and date-time start/stop times.
    * `libimagnotes` does not longer wrap store types but extend them.
    * `imag-notes` uses positional arguments now.
    * `libimagentrylist` does not export a CLI helper module anymore.

* Stats
    * ~325 commits
    * 82 merge-commits / 243 non-merge commits
    * 2 contributors
    * 447 files changed
    * 9749 insertions(+) / 7806 deletions(-) (Surely because of the
      reorganization of the entire codebase)

## 0.3.0

<small>
    Note: As this file was written _after_ the 0.3.0 release, we simply list the
    merges here instead of explaining what changed.
</small>

* Merges
    * [d14c972](http://git.imag-pim.org/imag/commit/?id=d14c9720e7ff4982ec5c13e011c8c27f3e92ea10)
      matthiasbeyer/release-commits-import
    * [f6a1c7d](http://git.imag-pim.org/imag/commit/?id=f6a1c7d56f1f559214a97d65dd1870e9f9906d71)
      matthiasbeyer/make-check
    * [0404b24](http://git.imag-pim.org/imag/commit/?id=0404b24333f7f41cb6821fd11003260ec45799af)
      matthiasbeyer/update-deps
    * [85e95d1](http://git.imag-pim.org/imag/commit/?id=85e95d142cc40f18df0da6a08e07ce6873394516)
      matthiasbeyer/readme-rewrite
    * [fa64c2d](http://git.imag-pim.org/imag/commit/?id=fa64c2d27dc4b9afbfa7d077ed7821e7688e0339)
      matthiasbeyer/libimagstore/store-id-cmp-without-base
    * [0a04081](http://git.imag-pim.org/imag/commit/?id=0a040815993803defe79749786adbd01f40b79b2)
      matthiasbeyer/cargo-rustc-codegen-units
    * [a4db420](http://git.imag-pim.org/imag/commit/?id=a4db420fdb43186258a7dc08ffb597a82d11f32a)
      matthiasbeyer/cargo-workspaces
    * [8bacdb4](http://git.imag-pim.org/imag/commit/?id=8bacdb49b9d2826bcf6ab5773ba09c46913ba5a9)
      matthiasbeyer/libimagref/remove-unused
    * [13c57aa](http://git.imag-pim.org/imag/commit/?id=13c57aa0cea1071accb00248f28537bf4288af13)
      matthiasbeyer/imag-link/reduce-unwraps
    * [e70fdc6](http://git.imag-pim.org/imag/commit/?id=e70fdc63c8f566039cf3f1afa910fa1a47430415)
      matthiasbeyer/libimagentrytag/remove-impl-tagable-on-fle
    * [1db063f](http://git.imag-pim.org/imag/commit/?id=1db063f3343ccb8d7a2ea2f1c3acf8eb24d39162)
      Stebalien/master
    * [a6a7e43](http://git.imag-pim.org/imag/commit/?id=a6a7e43b39979a276243807ac7569ef04e13f9db)
      mario-kr/add_shell-completion
    * [002c50a](http://git.imag-pim.org/imag/commit/?id=002c50a39e2a4e9426b0f8cc4bc7cc0d7ed8d599)
      matthiasbeyer/clap-completion
    * [b210b0e](http://git.imag-pim.org/imag/commit/?id=b210b0ec3edfc6269baedc2791d780b169975877)
      matthiasbeyer/libimagstore/entry-eq
    * [fe1c577](http://git.imag-pim.org/imag/commit/?id=fe1c5779634d22913db78b44717559b5a4e7c53f)
      matthiasbeyer/libimagstore/extract-toml-functionality
    * [4ca560a](http://git.imag-pim.org/imag/commit/?id=4ca560af7ff82d4dc9fabc5ef9abc579f288d3d7)
      matthiasbeyer/travis-use-old-rustc
    * [0310c21](http://git.imag-pim.org/imag/commit/?id=0310c2176f342fb42a55b5b3f025843bb8cf6a49)
      rnestler/libimagdiary/refactor_from_store_id
    * [9714028](http://git.imag-pim.org/imag/commit/?id=9714028cf3c6c3a1e6b32704030c52d2d82954c1)
      matthiasbeyer/clap-recommend-versions
    * [2003efd](http://git.imag-pim.org/imag/commit/?id=2003efd70646e121865cf12c4183c37388ad48f3)
      matthiasbeyer/imag-mail/init
    * [7c7aad9](http://git.imag-pim.org/imag/commit/?id=7c7aad9ea4578ac45e17895f73cf55d9a966b767)
      matthiasbeyer/libimagentrylink/fix-docu-typo
    * [0dd8498](http://git.imag-pim.org/imag/commit/?id=0dd849863fbca535525c136a02e47a3f8c394854)
      matthiasbeyer/update-deps
    * [9375c71](http://git.imag-pim.org/imag/commit/?id=9375c71292e8ec57bb6c970992f9f3aa33d54786)
      matthiasbeyer/makefile-check-outdated
    * [23a80ee](http://git.imag-pim.org/imag/commit/?id=23a80ee47f279fa1f5cd70ffab81e82629322dc8)
      matthiasbeyer/imag-link/external-link-remove-arg
    * [4a821d7](http://git.imag-pim.org/imag/commit/?id=4a821d7b196a4d478cdf51520c4c2939aaa06a81)
      matthiasbeyer/rust-beta-remove-top-level-cargotoml
    * [0cf5640](http://git.imag-pim.org/imag/commit/?id=0cf564091ece3c12ab83b1c5bb8555b1149c3d21)
      mario-kr/add_workspace-support
    * [c96e129](http://git.imag-pim.org/imag/commit/?id=c96e129b40c34a0fd99df3a9bf32285337885a4a)
      matthiasbeyer/libimagrt/logger-pub
    * [2c4946a](http://git.imag-pim.org/imag/commit/?id=2c4946a82c8eca33b619aeffc8a264566278658f)
      matthiasbeyer/remove-for-focus-shift
    * [9d7a26b](http://git.imag-pim.org/imag/commit/?id=9d7a26ba3ac42aa89670d032c97e5500ebde0828)
      matthiasbeyer/libimagrt/dbg-fileline-opt
    * [6dbecbd](http://git.imag-pim.org/imag/commit/?id=6dbecbd397de15727773857121282356cd98986d)
      matthiasbeyer/libimagrt/config-types-pub
    * [03a90c9](http://git.imag-pim.org/imag/commit/?id=03a90c9bf9ae4558842b30d0bce2968879e6efa8)
      matthiasbeyer/cleanup-bash-compl-gen
    * [6f564b5](http://git.imag-pim.org/imag/commit/?id=6f564b5223f9ec811f21081db31e16eb77d6d634)
      matthiasbeyer/love-to-defaultnix
    * [ce36b38](http://git.imag-pim.org/imag/commit/?id=ce36b38aa9ef25f8a0cc8f21860102b9950566a2)
      matthiasbeyer/fix-imag-bin-build
    * [cd684b0](http://git.imag-pim.org/imag/commit/?id=cd684b04ab696e7456d6ba6e51ebabcf96cb7f0f)
      matthiasbeyer/travis-opt
    * [636bfbb](http://git.imag-pim.org/imag/commit/?id=636bfbb768f23c92d581ab660fcaa88927c859b1)
      matthiasbeyer/imag-link/list-internal-only
    * [1e3193e](http://git.imag-pim.org/imag/commit/?id=1e3193ebb2028478aa26efd1b69697cddf00914f)
      matthiasbeyer/imag-ruby
    * [71e1a4c](http://git.imag-pim.org/imag/commit/?id=71e1a4cd61ad7748a252c77bb9c9eaa4ab01934e)
      matthiasbeyer/libimagerror/fix-warnings
    * [b03d1b5](http://git.imag-pim.org/imag/commit/?id=b03d1b562da4649d56411d21671ec5fc659ff65a)
      matthiasbeyer/libimagstore/fix-warnings
    * [0a417aa](http://git.imag-pim.org/imag/commit/?id=0a417aa3c6b599f067284ed1628d7e3fc5e9a44e)
      matthiasbeyer/libimagruby/fix-warnings
    * [55ea7f8](http://git.imag-pim.org/imag/commit/?id=55ea7f8228481a4859b1c2d34f9347b7cd35ee8e)
      matthiasbeyer/readme-updates
    * [ddc49de](http://git.imag-pim.org/imag/commit/?id=ddc49de0c315f10d7dd085aed3dbd3b88d78343b)
      matthiasbeyer/libimagruby/fix-macro
    * [df0fa43](http://git.imag-pim.org/imag/commit/?id=df0fa438c578bcf51ada250aea7eda7aed5ded89)
      matthiasbeyer/imag-tag/remove-warning
    * [3c7edcf](http://git.imag-pim.org/imag/commit/?id=3c7edcfb501d1f88df4a5e1d47c8bc1ac441cb20)
      matthiasbeyer/update-regex
    * [15b3567](http://git.imag-pim.org/imag/commit/?id=15b356773f7536dabc8346f79d9887016188e4b1)
      matthiasbeyer/workspace-fix-missing-doc
    * [6585677](http://git.imag-pim.org/imag/commit/?id=6585677d3147d338b271fb6477ea324694fa5454)
      matthiasbeyer/libimagentryfilter/remove-unused-import
    * [2ca89b7](http://git.imag-pim.org/imag/commit/?id=2ca89b73291a33d8209890b762343460ccc68604)
      matthiasbeyer/workspace-fix
    * [63ffbf6](http://git.imag-pim.org/imag/commit/?id=63ffbf62de95dfed62d3c41588472c3c8056a5a6)
      matthiasbeyer/libimagstore/eliminate-header-type
    * [3ffedec](http://git.imag-pim.org/imag/commit/?id=3ffedec8b8d3c785166021543ff69c15c246632c)
      matthiasbeyer/remove-warnings
    * [4d1282d](http://git.imag-pim.org/imag/commit/?id=4d1282d1631e8ff50d9e4ab5bc01590649c712d7)
      matthiasbeyer/libimagruby/impl-retrieve-for-mod
    * [c43538d](http://git.imag-pim.org/imag/commit/?id=c43538d517855d5d8f6b961de8465ba942917350)
      matthiasbeyer/ruby-build-setup
    * [2beb795](http://git.imag-pim.org/imag/commit/?id=2beb79581deca5c9b37cdfd01834f29c3934ecd5)
      matthiasbeyer/revert-871-ruby-build-setup
    * [b67b6f5](http://git.imag-pim.org/imag/commit/?id=b67b6f53434eb551fa45b50d616f805dd39bc96b)
      matthiasbeyer/libimagstore/doc
    * [dc1c473](http://git.imag-pim.org/imag/commit/?id=dc1c4733772da76d41af5c5a4e42c4927fb1c9cc)
      matthiasbeyer/libimag-todos
    * [bb126d5](http://git.imag-pim.org/imag/commit/?id=bb126d50a93dc11bcd26fb6601b16cd07cdbfcae)
      matthiasbeyer/libimagruby/api-brush
    * [b50334c](http://git.imag-pim.org/imag/commit/?id=b50334c10f0d72ac65981d5e4dc28e34e9edc382)
      matthiasbeyer/libimagrt/doc
    * [54655b9](http://git.imag-pim.org/imag/commit/?id=54655b9bce1ea4367396af5de0c2aa7376a05cea)
      matthiasbeyer/libimaginteraction/unpub-fn
    * [e33e5d2](http://git.imag-pim.org/imag/commit/?id=e33e5d287b961ff79d638d3369ea39b9e8c063a0)
      matthiasbeyer/libimagannotation/init
    * [f3af9e0](http://git.imag-pim.org/imag/commit/?id=f3af9e0ac41c3d0eb3820a04f20630af79160804)
      matthiasbeyer/clap-bump
    * [ef07c2c](http://git.imag-pim.org/imag/commit/?id=ef07c2cba946d930e72bd5576b738987e09c2593)
      matthiasbeyer/libimagstore/verify-panic
    * [a0f581b](http://git.imag-pim.org/imag/commit/?id=a0f581b3426eacf492aa2c9dc914abf78f9fd701)
      matthiasbeyer/libimagentryedit/dont-impl-for-fle
    * [84bcdc6](http://git.imag-pim.org/imag/commit/?id=84bcdc68b75188b818a6d384e02818ee2324cfba)
      matthiasbeyer/libimagnote/note-doesnt-need-to-be-tagable
    * [85cb954](http://git.imag-pim.org/imag/commit/?id=85cb954b9fe907e9efc6d87744b0262f6f1960fc)
      matthiasbeyer/less-fold-more-defresult
    * [c4bd98a](http://git.imag-pim.org/imag/commit/?id=c4bd98a48fe56625179ce2ecfad403164f0a32b8)
      mario-kr/makefile_use_workspaces
    * [3a0166b](http://git.imag-pim.org/imag/commit/?id=3a0166ba7c301b92cc0ccdbec137a42345963dd9)
      matthiasbeyer/libimagruby/error-types
    * [5d4ef8e](http://git.imag-pim.org/imag/commit/?id=5d4ef8ed7f243c73b514378fa9693e92295065cd)
      matthiasbeyer/libimagstore/non-consuming-update
    * [e615ec0](http://git.imag-pim.org/imag/commit/?id=e615ec040f57b9b4b20f58d623fe32b0e6588257)
      matthiasbeyer/add-libruby-travis-dep
    * [63faf06](http://git.imag-pim.org/imag/commit/?id=63faf06bc2774ac309c3783a8eefb286de30e570)
      matthiasbeyer/fix-warnings
    * [6f6368e](http://git.imag-pim.org/imag/commit/?id=6f6368ed2f120fafce94ec07e3abd5061242eb64)
      matthiasbeyer/travis-fixes
    * [9396acc](http://git.imag-pim.org/imag/commit/?id=9396accc28a63d280b61e2206320a6b1afeafdc3)
      matthiasbeyer/superceed-898
    * [6fa281a](http://git.imag-pim.org/imag/commit/?id=6fa281a1a4e0c99b4bcb5a95f016018ef7453cd3)
      matthiasbeyer/redo-ruby-build-setup
    * [5b93f38](http://git.imag-pim.org/imag/commit/?id=5b93f3848cb60ae76a450f79b6f3bd984847db0e)
      matthiasbeyer/libimagstore/storeid-exists-interface-result
    * [03f17b8](http://git.imag-pim.org/imag/commit/?id=03f17b8a1c71efc385b645c0db74a5e2f6b9dfd9)
      matthiasbeyer/libimagentrylink/annotations
    * [25a3518](http://git.imag-pim.org/imag/commit/?id=25a35183dd29051a159475f4a18d10de5051387c)
      matthiasbeyer/libimagentrylink/fix-exists
    * [7e3c946](http://git.imag-pim.org/imag/commit/?id=7e3c9467e7b95324f1bc34dc05fb8b33e2a26e90)
      matthiasbeyer/libimagutil/fix
    * [8eaead5](http://git.imag-pim.org/imag/commit/?id=8eaead5f52894fb5c46fe9e5fbefbbf1d80fd6de)
      matthiasbeyer/fix-build-quick
    * [241f975](http://git.imag-pim.org/imag/commit/?id=241f9752534c2518b160141509914ed7bb1d364e)
      matthiasbeyer/libimagentryedit/remove-unused-imports
    * [c74c26c](http://git.imag-pim.org/imag/commit/?id=c74c26ccd143d905c94ecf84ac423293b7170623)
      matthiasbeyer/fix-readme-links
    * [878162f](http://git.imag-pim.org/imag/commit/?id=878162f263b8d90dccbea8b1b82e96e005e04860)
      matthiasbeyer/libimagstore/store-id-tests
    * [1da56c6](http://git.imag-pim.org/imag/commit/?id=1da56c6d9df689150c94631bbb5147c36070b75c)
      matthiasbeyer/prepare-0.3.0
    * [4257ec1](http://git.imag-pim.org/imag/commit/?id=4257ec10268fef06d5888c48f9fc8f9e6f35c5ba)
      matthiasbeyer/update-toml
    * [a5857fa](http://git.imag-pim.org/imag/commit/?id=a5857fa64c949c9f2c9dea3036d870bd592272cc)
      matthiasbeyer/libimagstore/configuration-tests
    * [4ba1943](http://git.imag-pim.org/imag/commit/?id=4ba19430b754d47fc673164f7db7c1e4e619eb31)
      matthiasbeyer/add-dep-ismatch
    * [5ba2568](http://git.imag-pim.org/imag/commit/?id=5ba2568415615b7fcf3f2dce939ee2695bf498ff)
      asuivelentine/master
    * [dd24ce8](http://git.imag-pim.org/imag/commit/?id=dd24ce810a80222a625b5f24e6e2b7cb132a91c1)
      matthiasbeyer/revert-854
    * [bb9ff5b](http://git.imag-pim.org/imag/commit/?id=bb9ff5bfd824bd1c09f98fc9f77348965f7f1573)
      matthiasbeyer/remove-hooks
    * [08f43c8](http://git.imag-pim.org/imag/commit/?id=08f43c88111f90e5e2ac4980c572835d4a32fa8c)
      matthiasbeyer/update-toml-query
    * [16a12af](http://git.imag-pim.org/imag/commit/?id=16a12af873bdeae7eb2da94def40086fb239b1da)
      matthiasbeyer/libimagentrydate/init
    * [1b15d45](http://git.imag-pim.org/imag/commit/?id=1b15d45e7cf0e63e8370fbf779b12a9fce27412d)
      matthiasbeyer/libimagentrydate/fix-header-location
    * [4fff92e](http://git.imag-pim.org/imag/commit/?id=4fff92e7c02888567eae3aa34a63c19e9611daf9)
      matthiasbeyer/libimagmail/use-email-crate
    * [ef82b2a](http://git.imag-pim.org/imag/commit/?id=ef82b2ab415c7264109cf21d5c60f2a85340d627)
      matthiasbeyer/add-missing-license-header
    * [15b77ac](http://git.imag-pim.org/imag/commit/?id=15b77ac2c140dc14f6670a05e2b0f324165c8b2f)
      matthiasbeyer/libimagentrytag/clap-validators
    * [a9d2d7c](http://git.imag-pim.org/imag/commit/?id=a9d2d7c3545ab424cf1c6d2ebeea827616a924fe)
      matthiasbeyer/libimagstore/fs-memory-backend-as-dependency-injection
    * [c4d4fe9](http://git.imag-pim.org/imag/commit/?id=c4d4fe938937a2cb27d404fb1d026f234c50b9ac)
      matthiasbeyer/libimagstore/remove-todo-comment
    * [71e3d3d](http://git.imag-pim.org/imag/commit/?id=71e3d3d2d11219fdd1810c211708d8a321d45fa3)
      matthiasbeyer/libimagentrytag/validator-helper-enhancement
    * [bc95c56](http://git.imag-pim.org/imag/commit/?id=bc95c5615d8bac61757dabdc1e6148af626502ff)
      matthiasbeyer/libimagstore/fs-abstraction-pub
    * [f487550](http://git.imag-pim.org/imag/commit/?id=f487550f8174e4ad8ee69361b01b19acedf69cba)
      matthiasbeyer/libimagstore/storeid-local-part-altering
    * [cd99873](http://git.imag-pim.org/imag/commit/?id=cd99873f1700bbd6eb53f5d63ddea874750afb67)
      matthiasbeyer/libimagstore/io-backend
    * [e75c37f](http://git.imag-pim.org/imag/commit/?id=e75c37fbb2eaad7018c1ad18c227e82e67ec9629)
      matthiasbeyer/libimagstore/io-backend-knows-format
    * [d33b435](http://git.imag-pim.org/imag/commit/?id=d33b4350313364bab82c7e509bb9d6da219f5bb0)
      matthiasbeyer/libimagstore/all-entries
    * [f8ed679](http://git.imag-pim.org/imag/commit/?id=f8ed6794c2fc54ab4b61065835416a32eef74557)
      matthiasbeyer/libimagstore/backend-replacement
    * [2c97d6f](http://git.imag-pim.org/imag/commit/?id=2c97d6f1946d0f05bb95ca8d2ed09d093c4e6e92)
      matthiasbeyer/libimagstore/embellishments
    * [17bab5b](http://git.imag-pim.org/imag/commit/?id=17bab5b0b972fd9ea3198157f18e45d160679064)
      matthiasbeyer/libimagstore/fixes
    * [2b77064](http://git.imag-pim.org/imag/commit/?id=2b7706424a225e89e79b172ae07a6b12ee2ba74a)
      matthiasbeyer/libimagrt/fixes
    * [c9d03fc](http://git.imag-pim.org/imag/commit/?id=c9d03fc3c2eed003efac9cc1d805a51be69b7cb9)
      matthiasbeyer/update-travis
    * [22a4dc0](http://git.imag-pim.org/imag/commit/?id=22a4dc09293c21d34317e3b3f6b6c3a366ce1923)
      matthiasbeyer/libimagrt/cleanup
    * [b47972b](http://git.imag-pim.org/imag/commit/?id=b47972beddc66c07d1b0bfdfb3947f5392e917cb)
      matthiasbeyer/imag-store-dump
    * [1b88c22](http://git.imag-pim.org/imag/commit/?id=1b88c22decf168e2e02961c85cf5190ed44b6dc5)
      matthiasbeyer/libimagentrycategory/init
    * [7dea53c](http://git.imag-pim.org/imag/commit/?id=7dea53c6c00ad615ffcfb372859fcd61d08c4a21)
      matthiasbeyer/libimagannotation/add-doc
    * [c71b707](http://git.imag-pim.org/imag/commit/?id=c71b70702c30f587272fc2dbb20344bbcdeac872)
      matthiasbeyer/libimagannotation/add-is_annotation
    * [b3e7f09](http://git.imag-pim.org/imag/commit/?id=b3e7f095ce2b91255dd641e68009afba7582db53)
      matthiasbeyer/libimagtimetrack
    * [c75cfe4](http://git.imag-pim.org/imag/commit/?id=c75cfe4b608068fd65b4a2fa273a7b06f7ed51b6)
      matthiasbeyer/imag-link/fix-panic
    * [e80608c](http://git.imag-pim.org/imag/commit/?id=e80608c6097cac5a44d16a0d28ae7c1279c8a6a5)
      matthiasbeyer/libimagstore/fix-file-length-setting
    * [b4d0398](http://git.imag-pim.org/imag/commit/?id=b4d039833305565b9182d9d6ff4e162287a21fbf)
      matthiasbeyer/imag-link/export-consistency-check
    * [f041fb3](http://git.imag-pim.org/imag/commit/?id=f041fb3b1836ca7cd34c45b62c1abe2a69e53c5e)
      matthiasbeyer/fix-dep-rustc-version
    * [297eeb1](http://git.imag-pim.org/imag/commit/?id=297eeb1bd24a79ff29c55d5a591db05577b08cfd)
      matthiasbeyer/remove-nix-deps
    * [bee4e06](http://git.imag-pim.org/imag/commit/?id=bee4e0642689deb563dd92d02e2b211647db6a6d)
      irobert91/imag-link/rewrite-tests
    * [afc5d1f](http://git.imag-pim.org/imag/commit/?id=afc5d1f929a49c72e76dd0140570e21a982817a6)
      matthiasbeyer/update-toml-query
    * [58047d3](http://git.imag-pim.org/imag/commit/?id=58047d319a0aaa82fd9da7c4cc4857ca65ef53f6)
      matthiasbeyer/libimagtimetrack-to-libimagentrytimetrack
    * [3c07f47](http://git.imag-pim.org/imag/commit/?id=3c07f47c4ae6c9a74958200b018646233fce23fe)
      matthiasbeyer/libimagtimetrack/more-features
    * [3767d8d](http://git.imag-pim.org/imag/commit/?id=3767d8d38f81b2ec0382a3576811820179e3d249)
      matthiasbeyer/update-chrono
    * [c9360a4](http://git.imag-pim.org/imag/commit/?id=c9360a460abe0faf013d0101659bc594c5d0306c)
      matthiasbeyer/imag-link/test-utils-to-libimagutil
    * [e4f8d4e](http://git.imag-pim.org/imag/commit/?id=e4f8d4ec08cd506de10a5c01d6749bb9a993c603)
      matthiasbeyer/imag-tag/tests
    * [fc5bbc3](http://git.imag-pim.org/imag/commit/?id=fc5bbc3b9df91f4dd93bfa11ec0b71dddd593d2d)
      matthiasbeyer/libimagstore/glob-iterator-fix
    * [ec1c1e8](http://git.imag-pim.org/imag/commit/?id=ec1c1e8e3d40ec8c95947bdf66c448fbad5e6a40)
      matthiasbeyer/bin-refactor
    * [4b07c21](http://git.imag-pim.org/imag/commit/?id=4b07c21c34c047a7be2ae5041397842c811d80b3)
      matthiasbeyer/todo
    * [057d919](http://git.imag-pim.org/imag/commit/?id=057d9192398e19f03297c094d9fbaf8932cadfa0)
      matthiasbeyer/imag-timetrack
    * [0f436d5](http://git.imag-pim.org/imag/commit/?id=0f436d5b88f2794159c6837cc914bd5fa8bcce55)
      matthiasbeyer/doc-overhaul
    * [a1289cc](http://git.imag-pim.org/imag/commit/?id=a1289cc559d783f273d39316eb37c0c3b9ce5f7d)
      matthiasbeyer/update-readme
* Stats:
    * 127 merged branches
    * 8 contributors

## 0.2.0

* Complete rewrite of 0.1.0 with new architecture and "modules" instead of
  monolithic approach. Can be considered first version.

## 0.1.0

* Initial version, nothing special.
