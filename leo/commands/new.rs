// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use crate::{commands::Command, config::*, context::Context};
use leo_package::LeoPackage;

use anyhow::{anyhow, Result};
use std::{env::current_dir, fs};
use structopt::StructOpt;
use tracing::span::Span;

/// Create new Leo project
#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct New {
    #[structopt(name = "NAME", help = "Set package name")]
    name: String,
}

impl Command for New {
    type Input = ();
    type Output = ();

    fn log_span(&self) -> Span {
        tracing::span!(tracing::Level::INFO, "New")
    }

    fn prelude(&self, _: Context) -> Result<Self::Input> {
        Ok(())
    }

    fn apply(self, _: Context, _: Self::Input) -> Result<Self::Output> {
        // Check that the given package name is valid.
        let package_name = self.name;
        if !LeoPackage::is_package_name_valid(&package_name) {
            return Err(anyhow!("Invalid Leo project name"));
        }

        let username = read_username().ok();

        // Derive the package directory path.
        let mut path = current_dir()?;
        path.push(&package_name);

        // Verify the package directory path does not exist yet.
        if path.exists() {
            return Err(anyhow!("Directory already exists {:?}", path));
        }

        // Create the package directory
        fs::create_dir_all(&path).map_err(|err| anyhow!("Could not create directory {}", err))?;

        LeoPackage::initialize(&package_name, false, &path, username)?;

        Ok(())
    }
}
