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

use anyhow::{anyhow, bail};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, PartialEq, Debug)]
pub struct ItemArg {
   pub typ: String,
   pub name: String,
}

impl ItemArg {
   pub fn has_ref(&self) -> bool {
      self.typ.starts_with('&')
   }
}

#[derive(Default)]
pub struct ItemValue {
   pub fmt_str: String,
   pub args: Vec<ItemArg>,
}

impl ItemValue {
   pub fn parse(s: &str) -> anyhow::Result<Self> {
      let mut fmt = String::default();
      let mut args = Vec::default();
      let mut arg_buff = String::default();

      let mut is_var = false;
      for ch in s.chars() {
         if ch == '$' {
            arg_buff.clear();
            is_var = true;
            continue;
         }

         if !is_var {
            fmt.push(ch)
         } else {
            if ch == '{' {
               fmt.push(ch);
            } else {
               arg_buff.push(ch);
            }

            if ch == '}' {
               fmt.push(ch);
               arg_buff.pop();
               is_var = false;

               let mut typ = String::default();
               let mut name = String::default();

               let mut is_type = false;
               for ch in arg_buff.chars() {
                  if ch == ':' {
                     is_type = true;
                     continue;
                  }

                  if is_type {
                     typ.push(ch);
                  } else {
                     name.push(ch);
                  }
               }
               let mut item_arg = ItemArg { typ, name };
               if item_arg.typ.is_empty() {
                  item_arg.typ.push_str("&str");
               }
               args.push(item_arg)
            }
         }
      }

      Ok(Self { fmt_str: fmt, args })
   }

   pub fn has_ref(&self) -> bool {
      for a in &self.args {
         if a.has_ref() {
            return true;
         }
      }
      false
   }
}

#[derive(Default)]
pub struct Item {
   pub key: String,
   pub values: HashMap<String, ItemValue>,
   pub groups: HashMap<String, Self>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Local {
   pub root: Item,
}

impl Local {
   pub fn load(file: &Path) -> anyhow::Result<Self> {
      pub type Tr = HashMap<String, serde_json::Value>;

      let f = OpenOptions::new().read(true).open(file)?;
      let r = std::io::BufReader::new(f);
      let tr: Tr = serde_yaml::from_reader(r)?;

      if tr.len() != 1 {
         bail!("Wrong the first level size. Expected local name as a key!");
      }

      let mut root = Item::default();
      let values = tr.iter().next().unwrap();
      root.key = values.0.to_string();

      if values.1.is_object() {
         for (k, v) in values.1.as_object().unwrap() {
            Local::fill_item(k, v, &mut root)?;
         }
      } else {
         bail!("Unexpected first level value type: {:?}", values.1);
      }

      Ok(Self { root })
   }

   fn fill_item(key: &str, val: &serde_json::Value, item: &mut Item) -> anyhow::Result<()> {
      match val {
         serde_json::Value::String(s) => {
            item.values.insert(key.to_string(), ItemValue::parse(s)?);
         }
         serde_json::Value::Object(obj) => {
            let child = item
               .groups
               .entry(key.to_string())
               .or_insert_with(|| Item { key: key.to_string(), ..Item::default() });
            for (k, v) in obj {
               Local::fill_item(k, v, child)?;
            }
         }
         _ => bail!("Unexpected type: {:?}", val),
      }
      Ok(())
   }

   pub fn check_matching(&self, other: &Local) -> anyhow::Result<()> {
      for v in self.root.values.iter() {
         other.root.values.get(v.0.as_str()).ok_or_else(|| {
            anyhow!("The local [{}] does not have key: [{}]", other.root.key, v.0)
         })?;
      }

      for this_group in self.root.groups.iter() {
         let other_group = other.root.groups.get(this_group.0.as_str()).ok_or_else(|| {
            anyhow!("The local [{}] does not have group: [{}]", other.root.key, this_group.0)
         })?;

         for v in this_group.1.values.iter() {
            other_group.values.get(v.0.as_str()).ok_or_else(|| {
               anyhow!(
                  "The local [{}] does not have key: [{}:{}]",
                  other.root.key,
                  this_group.0,
                  v.0
               )
            })?;
         }
      }
      Ok(())
   }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
   use super::*;
   use crate::test_utils::{init_logger, test_assets_dir};

   #[test]
   fn test_parse_value() {
      let v = ItemValue::parse("hello world!").unwrap();
      assert_eq!("hello world!", v.fmt_str);
      assert!(v.args.is_empty());

      let v = ItemValue::parse("hello ${name:&str}!").unwrap();
      assert_eq!("hello {}!", v.fmt_str);
      assert_eq!(ItemArg { typ: "&str".to_string(), name: "name".to_string() }, v.args[0]);

      let v = ItemValue::parse("number ${val1:u32} - ${val2:u32}!").unwrap();
      assert_eq!("number {} - {}!", v.fmt_str);
      assert_eq!(ItemArg { typ: "u32".to_string(), name: "val1".to_string() }, v.args[0]);
      assert_eq!(ItemArg { typ: "u32".to_string(), name: "val2".to_string() }, v.args[1]);
   }

   #[test]
   fn test_load_local() {
      init_logger();
      let file = test_assets_dir(Some("en-EN.yml"));
      let local = Local::load(&file).unwrap();
      assert_eq!("en-EN", local.root.key);

      assert_eq!("hello world!", local.root.values.get("hello").unwrap().fmt_str);
      assert_eq!("hello {}!", local.root.values.get("greet").unwrap().fmt_str);
      assert_eq!("number {}!", local.root.values.get("count").unwrap().fmt_str);

      let group = local.root.groups.get("group_1").unwrap();
      assert_eq!("hello world from group!", group.values.get("hello").unwrap().fmt_str);
      assert_eq!("hello {} from group!", group.values.get("greet").unwrap().fmt_str);
      assert_eq!("number {} - {} from group!", group.values.get("count").unwrap().fmt_str);
   }

   #[test]
   fn test_matching() {
      init_logger();
      let local1 = Local::load(&test_assets_dir(Some("en-EN.yml"))).unwrap();
      let local2 = Local::load(&test_assets_dir(Some("matching/match.yml"))).unwrap();
      let local3 = Local::load(&test_assets_dir(Some("matching/not_match.yml"))).unwrap();
      assert_eq!("en-EN", local1.root.key);
      assert_eq!("ru1", local2.root.key);
      assert_eq!("ru2", local3.root.key);
      local1.check_matching(&local2).unwrap();

      assert!(local1.check_matching(&local3).is_err())
   }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
