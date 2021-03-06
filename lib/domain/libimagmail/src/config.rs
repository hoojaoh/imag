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

/// A struct representing a full mail configuration, required for working with this library
///
/// For convenience reasons, this implements Serialize and Deserialize, so it can be fetched from a
/// configuration file for example
///
/// # TODO
///
/// Figure out how to use handlebars with variables on this. Right now the support for that is not
/// implemented yet.
///
#[derive(Serialize, Deserialize, Debug)]
pub struct MailConfig {
    default_account  : String,
    accounts         : Vec<MailAccountConfig>,
    fetchcommand     : MailCommand,
    postfetchcommand : Option<MailCommand>,
    sendcommand      : MailCommand,
    postsendcommand  : Option<MailCommand>,
}

impl MailConfig {
    pub fn default_account(&self) -> &String {
        &self.default_account
    }

    pub fn accounts(&self) -> &Vec<MailAccountConfig> {
        &self.accounts
    }

    pub fn account(&self, name: &str) -> Option<&MailAccountConfig> {
        self.accounts()
            .iter()
            .find(|a| a.name == name)
    }

    pub fn fetchcommand(&self) -> &MailCommand {
        &self.fetchcommand
    }

    pub fn postfetchcommand(&self) -> Option<&MailCommand> {
        self.postfetchcommand.as_ref()
    }

    pub fn sendcommand(&self) -> &MailCommand {
        &self.sendcommand
    }

    pub fn postsendcommand(&self) -> Option<&MailCommand> {
        self.postsendcommand.as_ref()
    }

    pub fn fetchcommand_for_account(&self, account_name: &str) -> &MailCommand {
        self.accounts()
            .iter()
            .find(|a| a.name == account_name)
            .and_then(|a| a.fetchcommand.as_ref())
            .unwrap_or_else(|| self.fetchcommand())
    }

    pub fn postfetchcommand_for_account(&self, account_name: &str) -> Option<&MailCommand> {
        self.accounts()
            .iter()
            .find(|a| a.name == account_name)
            .and_then(|a| a.postfetchcommand.as_ref())
            .or_else(|| self.postfetchcommand())
    }

    pub fn sendcommand_for_account(&self, account_name: &str) -> &MailCommand {
        self.accounts()
            .iter()
            .find(|a| a.name == account_name)
            .and_then(|a| a.sendcommand.as_ref())
            .unwrap_or_else(|| self.sendcommand())
    }

    pub fn postsendcommand_for_account(&self, account_name: &str) -> Option<&MailCommand> {
        self.accounts()
            .iter()
            .find(|a| a.name == account_name)
            .and_then(|a| a.postsendcommand.as_ref())
            .or_else(|| self.postsendcommand())
    }

}

/// A configuration for a single mail accounts
///
/// If one of the keys `fetchcommand`, `postfetchcommand`, `sendcommand` or `postsendcommand` is
/// not available, the implementation of the `MailConfig` will automatically use the global
/// configuration if applicable.
#[derive(Serialize, Deserialize, Debug)]
pub struct MailAccountConfig {
    pub name             : String,
    pub outgoingbox      : PathBuf,
    pub draftbox         : PathBuf,
    pub sentbox          : PathBuf,
    pub maildirroot      : PathBuf,
    pub fetchcommand     : Option<MailCommand>,
    pub postfetchcommand : Option<MailCommand>,
    pub sendcommand      : Option<MailCommand>,
    pub postsendcommand  : Option<MailCommand>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MailCommand {
    command: String,
    env: Vec<String>,
    args: Vec<String>,
}

