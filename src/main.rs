// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

mod fnv1_hash;
mod library;
mod md_content;
mod prompt;

use build_html as html;
use library::Library;
use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let dir = env::current_dir()?;
    let lib = Library::scan()?;
    lib.save("./whim.toml")?;

    Ok(())
}
