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

#[allow(unused)]
use leo_asg::{new_context, Asg, AsgContext};
use leo_ast::{Ast, ReconstructingReducer};
use leo_compiler::{CombineAstAsgDirector, CompilerOptions};
use leo_imports::ImportParser;
use leo_parser::parser;

use anyhow::{bail, Result};
use clap::{App, Arg};
use serde_json;

use std::{fs::File, io::prelude::*, path::PathBuf};

const TEST_PROGRAM_PATH: &str = "";

thread_local! {
    static THREAD_GLOBAL_CONTEXT: AsgContext<'static> = {
        let leaked = Box::leak(Box::new(leo_asg::new_alloc_context()));
        leo_asg::new_context(leaked)
    }
}

pub fn thread_leaked_context() -> AsgContext<'static> {
    THREAD_GLOBAL_CONTEXT.with(|f| *f)
}

struct TypeInferenceCombiner;
impl ReconstructingReducer for TypeInferenceCombiner {}

impl Default for TypeInferenceCombiner {
    fn default() -> Self {
        Self {}
    }
}

fn write_ast(ast: Ast, file: &str) -> Result<()> {
    let program = ast.into_repr();
    serde_json::to_writer_pretty(&File::create(file)?, &program)?;
    Ok(())
}

fn main() -> Result<()> {
    let matches = App::new("Leo Stages")
        .version("1.0")
        .about("Prints the Ast at different compiler stages. Does not acually compile/run the leo code.")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .help("Sets the path to the leo file.")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Writes all stages to json files(named after the stages)."),
        )
        .arg(
            Arg::with_name("initial")
                .short("i")
                .long("initial")
                .help("Writes the initially parsed ast to a initial.json file."),
        )
        .arg(
            Arg::with_name("canonicalization")
                .short("c")
                .long("canonicalize")
                .help("Writes the canonicalized ast to a canonicalization.json file."),
        )
        .arg(
            Arg::with_name("inference")
                .short("i")
                .long("inference")
                .help("Writes the type inferenced ast to a type_inference.json file."),
        )
        .get_matches();

    let test_program_file_path = PathBuf::from(TEST_PROGRAM_PATH);

    let file = matches.value_of("file");
    let mut file_string = String::new();

    match file {
        Some(file_str) => {
            let mut file = File::open(file_str)?;
            file.read_to_string(&mut file_string)?;
        }
        None => bail!("Please provide file path."),
    };

    let mut ast = Ast::new(parser::parse(
        test_program_file_path.to_str().expect("unwrap fail"),
        &file_string,
    )?);

    if matches.is_present("all") || matches.is_present("initial") {
        write_ast(ast.clone(), "initial.json")?;
    }

    ast.canonicalize()?;
    if matches.is_present("all") || matches.is_present("canonicalization") {
        write_ast(ast.clone(), "canonicalization.json")?;
    }

    let program = ast.clone().into_repr();
    let asg = Asg::new(thread_leaked_context(), &program, &mut ImportParser::default())?;

    let new_ast = Ast::new(
        CombineAstAsgDirector::new(TypeInferenceCombiner::default(), CompilerOptions::default())
            .reduce_program(&ast.clone().into_repr(), &asg.into_repr())?,
    );
    if matches.is_present("all") || matches.is_present("inference") {
        write_ast(new_ast.clone(), "type_inference.json")?;
    }

    Ok(())
}
