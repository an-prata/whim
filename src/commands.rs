// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use crate::{
    library::Library,
    prompt::{self, PromptItem},
};
use std::{error, process};

const LIBRARY_FILE: &str = ".whim.ron";
const HTML_PATH: &str = "./whim-build";

pub fn new() -> Result<(), Box<dyn error::Error>> {
    let lib = Library::scan()?;

    match lib.documents().len() > 0 {
        true => {
            println!(
                "whim found {} markdown documents in the current directory:",
                lib.documents().len()
            );

            for doc in lib.documents().keys() {
                println!("    {}", doc);
            }
        }
        false => {
            println!("whim found no markdown documents in the current directory")
        }
    }

    let yn = prompt::Yes::from_prompt(
        format!(
            "create a new library with {} documents",
            lib.documents().len()
        ),
        Some('?'),
    )?;

    match yn {
        prompt::Yes::Yes => {
            lib.save(LIBRARY_FILE)?;
            return Ok(());
        }
        prompt::Yes::No => Ok(()),
    }
}

pub fn update() -> Result<(), Box<dyn error::Error>> {
    let lib = open_lib();
    let docs = lib.changed_docs();

    match docs.len() {
        1.. => {
            println!("{} documents have changed:", docs.len());

            for d in docs.clone() {
                println!("    {}", d);
            }

            let yn =
                prompt::Yes::from_prompt(format!("update {} documents", docs.len()), Some('?'))?;

            match yn {
                prompt::Yes::Yes => {
                    lib.update()?.save(LIBRARY_FILE)?;
                    println!("updated {} documents", docs.len());
                    Ok(())
                }
                prompt::Yes::No => {
                    println!("updated 0 documents");
                    Ok(())
                }
            }
        }
        _ => {
            println!("no updates to make, 0 documents have changed");
            return Ok(());
        }
    }
}

#[inline]
fn open_lib() -> Library {
    match Library::open(LIBRARY_FILE) {
        Ok(l) => l,
        Err(e) => {
            println!("whim could not open a library in the current directory, you may need to create one with `whim new`");
            process::exit(0);
        }
    }
}
