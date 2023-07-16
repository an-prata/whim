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
use library::Library;
use prompt::PromptItem;
use std::{env, error::Error};

const NEW_COMMAND: &str = "new";
const UPDATE_COMMAND: &str = "update";
const SCAN_COMMAND: &str = "scan";
const ADD_COMMAND: &str = "add";

fn main() -> Result<(), Box<dyn Error>> {
    let cmd_new = Command(NEW_COMMAND.into());
    let cmd_update = Command(UPDATE_COMMAND.into());
    let cmd_scan = Command(SCAN_COMMAND.into());
    let cmd_add = Command(ADD_COMMAND.into());

    let args = match ArgsParser::new(env::args())
        .command(cmd_new.clone())
        .command(cmd_update.clone())
        .command(cmd_scan.clone())
        .command(cmd_add.clone())
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
