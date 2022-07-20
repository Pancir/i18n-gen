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

use crate::model::ItemArg;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Helper to make reusable output structures.
#[derive(Default)]
pub struct StructNames {
   counter: usize,
   names: HashMap<String, String>,
}

impl StructNames {
   pub fn get_or_add(&mut self, args: &[ItemArg]) -> (bool, &str) {
      let id = StructNames::create_id(args);
      let mut created = false;
      let res = self.names.entry(id).or_insert_with(|| {
         self.counter += 1;
         created = true;
         format!("S{}", self.counter)
      });
      (created, res)
   }

   fn create_id(args: &[ItemArg]) -> String {
      let mut s = String::with_capacity(256);
      for a in args {
         s.push_str(&a.name);
         s.push_str(&a.typ);
      }
      s
   }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn create_mod_name(s: &str) -> String {
   s.replace(' ', "_").replace('-', "_").to_lowercase()
}

pub fn create_fn_name(s: &str) -> String {
   s.replace(' ', "_").replace('-', "_").to_lowercase()
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Writes something like: `val1, val2, val3 ...`
pub fn seq_arg_names<'a>(prefix: &'a str, args: &'a [ItemArg]) -> impl std::fmt::Display + 'a {
   struct Out<'a> {
      prefix: &'a str,
      args: &'a [ItemArg],
   }
   impl<'a> std::fmt::Display for Out<'a> {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
         for (i, a) in self.args.iter().enumerate() {
            if i > 0 {
               write!(f, ", {}{}", self.prefix, a.name)?;
            } else {
               write!(f, "{}{}", self.prefix, a.name)?;
            }
         }
         Ok(())
      }
   }
   Out::<'a> { prefix, args }
}

/// Writes something like: `u32, u32, &str ...`
pub fn seq_arg_types<'a>(args: &'a [ItemArg]) -> impl std::fmt::Display + 'a {
   struct Out<'a> {
      args: &'a [ItemArg],
   }
   impl<'a> std::fmt::Display for Out<'a> {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
         for (i, a) in self.args.iter().enumerate() {
            if i > 0 {
               write!(f, ", {}", a.typ)?;
            } else {
               write!(f, "{}", a.typ)?;
            }
         }
         Ok(())
      }
   }
   Out::<'a> { args }
}

/// Writes something like: `val1: u32, val2: u32, val3: &str ...`
pub fn seq_args<'a>(life_time_name: &'a str, args: &'a [ItemArg]) -> impl std::fmt::Display + 'a {
   struct Out<'a> {
      life_time_name: &'a str,
      args: &'a [ItemArg],
   }
   impl<'a> std::fmt::Display for Out<'a> {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
         for (i, a) in self.args.iter().enumerate() {
            if i > 0 {
               if a.has_ref() {
                  write!(f, ", {}: &{} {}", a.name, self.life_time_name, &a.typ[1..])?;
               } else {
                  write!(f, ", {}: {}", a.name, a.typ)?;
               }
            } else if a.has_ref() {
               write!(f, "{}: &{} {}", a.name, self.life_time_name, &a.typ[1..])?;
            } else {
               write!(f, "{}: {}", a.name, a.typ)?;
            }
         }
         Ok(())
      }
   }
   Out::<'a> { life_time_name, args }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Writes something like: `val1: u32, val2: u32, val3: &str ...`
pub fn seq_struct_members<'a>(args: &'a [ItemArg]) -> impl std::fmt::Display + 'a {
   struct Out<'a> {
      args: &'a [ItemArg],
   }
   impl<'a> std::fmt::Display for Out<'a> {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
         for a in self.args {
            if !a.has_ref() {
               write!(f, "pub {}: {}, ", a.name, a.typ)?;
            } else {
               write!(f, "pub {}: &'a {}, ", a.name, &a.typ[1..])?;
            }
         }
         Ok(())
      }
   }
   Out::<'a> { args }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn join_tree_path<'a>(path: &'a [String], sep: &'a str) -> impl std::fmt::Display + 'a {
   struct Out<'a> {
      path: &'a [String],
      sep: &'a str,
   }
   impl<'a> std::fmt::Display for Out<'a> {
      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
         for p in self.path {
            write!(f, "{}{}", p, self.sep)?;
         }
         Ok(())
      }
   }
   Out::<'a> { path, sep }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
