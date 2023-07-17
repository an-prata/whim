// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

mod args;
mod commands;
mod fnv1_hash;
mod href;
mod library;
mod md_content;
mod prompt;

use args::{ArgsParser, Command};
use std::{env, error::Error};

const NEW_COMMAND: &str = "new";
const UPDATE_COMMAND: &str = "update";
const SCAN_COMMAND: &str = "scan";
const ADD_COMMAND: &str = "add";
const BUILD_COMMAND: &str = "build";

fn main() -> Result<(), Box<dyn Error>> {
    let cmd_new = Command(NEW_COMMAND.into());
    let cmd_update = Command(UPDATE_COMMAND.into());
    let cmd_scan = Command(SCAN_COMMAND.into());
    let cmd_add = Command(ADD_COMMAND.into());
    let cmd_build = Command(BUILD_COMMAND.into());

    let args = match ArgsParser::new(env::args())
        .command(cmd_new)
        .command(cmd_update)
        .command(cmd_scan)
        .command(cmd_add.clone())
        .command(cmd_build.clone())
        .parse()
    {
        Ok(v) => v,
        Err(_) => {
            print_help();
            return Ok(());
        }
    };

    let command = {
        let cmds = args.commands();

        if cmds.len() > 1 {
            println!("Only singlular commands permitted.");
            return Ok(());
        }

        cmds[0].clone()
    };

    match &*command.0 {
        NEW_COMMAND => return commands::new(),
        UPDATE_COMMAND => return commands::update(),
        SCAN_COMMAND => return commands::scan(),
        ADD_COMMAND => {
            return commands::add(match &args.command_parameters(cmd_add).unwrap()[0] {
                args::Value::String(s) => s.clone(),
                _ => unreachable!(),
            })
        }
        BUILD_COMMAND => {
            return commands::build(match &args.command_parameters(cmd_build).unwrap()[0] {
                args::Value::String(s) => s.clone(),
                _ => unreachable!(),
            })
        }
        _ => (),
    };

    Ok(())
}

fn print_help() {
    println!(
        "\
        whim\n\
        \n\
        Usage: whim [COMMAND]\n\
        \n\
        Commands:\n\
        \tnew      Creates new library in the current directory.\n\
        \tupdate   Updates the library in the current directory.\n\
        \tscan     Scans the directory for new files.\n\
        \tadd      Add a document.\
        "
    )
}
