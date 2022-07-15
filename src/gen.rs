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

use crate::codegen::generate_code;
use crate::model::Local;
use anyhow::bail;
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn generate(local_dir: &Path, mod_dir: &Path) -> anyhow::Result<()> {
   let default_file = local_dir.join("!default.yml");
   if !default_file.exists() {
      bail!("There is not !default.yml file in the dir: {}", local_dir.display());
   }

   let default = Local::load(&default_file)?;
   println!("Loaded [{}]", default_file.display());

   let mut locals = Vec::<Local>::with_capacity(64);

   for entry in std::fs::read_dir(local_dir)? {
      let entry = entry?;
      if !entry.metadata().unwrap().is_dir() {
         let path = entry.path();
         if let Some(ext) = path.extension() {
            if ext == "yml" {
               locals.push(Local::load(&path)?);
            }
         }
      }
   }

   locals.retain(|v| v.root.key != default.root.key);

   for l in &locals {
      default.check_matching(&l)?;
   }

   generate_code(&default, &locals, mod_dir)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
   use super::*;
   use crate::test_utils::{init_logger, test_assets_dir, test_gen_dir};

   #[test]
   fn test_generate() {
      init_logger();
      generate(&test_assets_dir(None), &test_gen_dir(None)).unwrap()
   }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
