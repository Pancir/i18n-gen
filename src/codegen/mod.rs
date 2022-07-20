/*
**  Copyright 2022 The library developers and contributors
**
**  Redistribution and use in source and binary forms, with or without
**  modification, are permitted provided that the following conditions are met:
**
**  1.Redistributions of source code must retain the above copyright notice, this
**    list of conditions and the following disclaimer.
**  2.Redistributions in binary form must reproduce the above copyright notice,
**    this list of conditions and the following disclaimer in the documentation
**    and / or other materials provided with the distribution.
**  3.Neither the name of the Library creator nor the names of its contributors
**    may be used to endorse or promote products derived from this software
**    without specific prior written permission.
**
**  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
**  ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
**  WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
**  DISCLAIMED.IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR
**  ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
**  (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
**  LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
**  ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
**  (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
**  SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::model::Local;
use anyhow::bail;
use std::path::Path;

mod gen;
pub(crate) mod helpers;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Generator configuration.
pub struct Config<'a> {
   /// File name without extension for default local.
   ///
   /// It is also a main scheme template.
   /// Scheme of other locals will be compared to this one.
   ///
   /// Default: "en-EN"
   pub default_local_file: &'static str,

   /// It true then the `allow(dead_code)` attr will be written on top of the generated file.
   ///
   /// Set it to false and this can help you to find text that is not used
   /// if the generated `tr` module is not public.
   ///
   /// Default: true
   pub dead_code_attr: bool,

   /// List of imports.
   ///
   /// Your custom types must be imported with `use` before they are used.
   /// Example: Some(&\["crate::DateFormatter"\]) This will be printed as
   /// `use crate::DateFormatter;` at the top of generated file.
   pub imports: &'a [&'a str],
}

impl<'a> Default for Config<'a> {
   fn default() -> Self {
      Self { default_local_file: "en-EN", dead_code_attr: true, imports: &[] }
   }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Generate translation module using path to dir.
///
/// # Notes
/// The function walk only one dir level. I.e. not walking subdirs.
pub fn generate(local_dir: &Path, mod_dir: &Path, config: Config<'_>) -> anyhow::Result<()> {
   let mut locals = Vec::<Local>::with_capacity(64);
   let mut default_local_key = String::with_capacity(64);

   for entry in std::fs::read_dir(local_dir)? {
      let entry = entry?;
      if !entry.metadata().unwrap().is_dir() {
         let path = entry.path();
         if let Some(ext) = path.extension() {
            if ext == "yml" {
               // TODO may be it is a good idea to check if file name matches local code key.
               let local = Local::load(&path)?;
               if path.file_name().unwrap().to_str().unwrap().starts_with(config.default_local_file)
               {
                  default_local_key = local.root.key.clone();
               }
               locals.push(local);
            }
         }
      }
   }

   if default_local_key.is_empty() {
      bail!(
         "The default local file [{}.yml] in the directory [{}] is not found!",
         &config.default_local_file,
         local_dir.display()
      );
   }

   let mut default_position: usize = 0;
   for (i, loc) in locals.iter().enumerate() {
      if loc.root.key == default_local_key {
         default_position = i;
      }

      if i < locals.len() - 1 {
         for other in &locals[(i + 1)..] {
            if other.root.key == loc.root.key {
               bail!("More than one local with the same code [{}] is found!", other.root.key);
            }
         }
      }
   }

   if default_position != 0 {
      locals.swap(default_position, 0)
   }

   let default = locals.first().unwrap();
   for l in &locals[1..] {
      default.check_matching(l)?;
   }

   gen::generate_code(&locals, mod_dir, config)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
   use super::*;
   use crate::test_utils::{init_logger, test_assets_dir, test_gen_dir};

   #[test]
   fn test_generate() {
      init_logger();
      generate(&test_assets_dir(None), &test_gen_dir(None), Config::default()).unwrap()
   }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
