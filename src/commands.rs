// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use crate::{
    library::Library,
    prompt::{self, PromptItem},
};
use std::{error, process};

const LIBRARY_FILE: &str = ".whim.ron";

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

            let yn = prompt::Yes::from_prompt(
                format!("update {} documents in library", docs.len()),
                Some('?'),
            )?;

            match yn {
                prompt::Yes::Yes => {
                    let len = docs.len();
                    lib.update()?.save(LIBRARY_FILE)?;
                    println!("updated {} documents in library", len);
                    Ok(())
                }
                prompt::Yes::No => {
                    println!("updated 0 documents in library");
                    Ok(())
                }
            }
        }
        _ => {
            println!("no updates to make");
            return Ok(());
        }
    }
}

pub fn scan() -> Result<(), Box<dyn error::Error>> {
    let mut lib = open_lib();
    let docs = lib.scan_for_new()?;

    match docs.len() {
        1.. => {
            println!("found {} documents not in the library:", docs.len());

            for doc in docs.clone() {
                println!("    {}", doc);
            }

            let yn = prompt::Yes::from_prompt(
                format!("add {} documents to library", docs.len()),
                Some('?'),
            )?;

            match yn {
                prompt::Yes::Yes => {
                    for doc in docs.clone() {
                        match lib.add_document(doc.as_ref()) {
                            Ok(_) => println!("    added {}", doc),
                            Err(_) => println!("    failed to add {}", doc),
                        }
                    }

                    match lib.save(LIBRARY_FILE) {
                        Ok(_) => println!("added {} documents to library", docs.len()),
                        Err(_) => println!("could not update library with new documents"),
                    }

                    Ok(())
                }
                prompt::Yes::No => todo!(),
            }
        }
        _ => {
            println!("found no documents not already in library");
            Ok(())
        }
    }
}

pub fn add(path: String) -> Result<(), Box<dyn error::Error>> {
    let mut lib = open_lib();

    match lib.add_document(path.clone()) {
        Ok(_) => (),
        Err(_) => {
            println!("could not add '{}'", path);
            return Ok(());
        }
    }

    match lib.save(LIBRARY_FILE) {
        Ok(_) => println!("added '{}'", path),
        Err(_) => println!("could not save library, add failed"),
    }

    Ok(())
}

pub fn build(path: String) -> Result<(), Box<dyn error::Error>> {
    let lib = open_lib();

    let lib_html = match lib.gen_html() {
        Ok(v) => v,
        Err(_) => {
            println!("could not read all documents for parsing");
            return Ok(());
        }
    };

    match lib_html.write(path.clone()) {
        Ok(_) => println!("wrote HTML to '{}'", path),
        Err(_) => println!("could not write HTML to '{}", path),
    }

    Ok(())
}

#[inline]
fn open_lib() -> Library {
    match Library::open(LIBRARY_FILE) {
        Ok(l) => l,
        Err(_) => {
            println!("whim could not open a library in the current directory, you may need to create one with `whim new`");
            process::exit(0);
        }
    }
}
