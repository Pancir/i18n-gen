use crate::gen_code::generate_code;
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
