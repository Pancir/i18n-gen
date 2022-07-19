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

use crate::codegen::helpers::{
   create_fn_name, create_mod_name, join_tree_path, seq_arg_names, seq_arg_types, seq_args,
   seq_struct_members, StructNames,
};
use crate::model::{Item, ItemValue, Local};
use crate::Config;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn generate_code(locals: &[Local], mod_dir: &Path, config: Config) -> anyhow::Result<()> {
   fn write_sep(r: &mut impl Write) -> anyhow::Result<()> {
      writeln!(
         r,
         "\n////////////////////////////////////////////////////////////////////////////////////////////////////\n"
      )?;
      Ok(())
   }
   //-------------------------

   let mod_file_path = mod_dir.join("tr.rs");
   let mut f =
      OpenOptions::new().read(true).write(true).truncate(true).create(true).open(mod_file_path)?;

   let mut struct_names = StructNames::default();
   let mut tree_path = Vec::<String>::with_capacity(8);
   if config.dead_code_attr {
      writeln!(f, "#![allow(dead_code)]")?;
   }
   writeln!(f, "#![allow(non_upper_case_globals)]")?;
   write_sep(&mut f)?;

   //-------------------------

   write!(f, "pub mod defines {{")?;
   write_pre_defined_traits(&mut f)?;
   write_pre_defined_structs(&mut f)?;
   for l in locals {
      write_structs(&mut f, &mut struct_names, &l.root)?;
   }
   writeln!(f, "}}")?;

   //-------------------------

   write_sep(&mut f)?;
   tree_path.clear();
   write_local_statics(&mut f, &mut tree_path, &mut struct_names, locals)?;

   //-------------------------

   tree_path.clear();
   write_local_hierarchy(
      &mut f,
      &mut tree_path,
      &mut struct_names,
      &locals.first().unwrap().root,
      locals,
   )?;
   write_global_static(&mut f, locals)?;

   //-------------------------

   write_sep(&mut f)?;
   for l in locals {
      tree_path.clear();
      tree_path.push(create_mod_name(&l.root.key));
      write_local_groups(&mut f, &mut tree_path, &mut struct_names, &l.root)?;
   }

   //-------------------------

   write_sep(&mut f)?;
   writeln!(f, "mod fmt {{")?;
   for l in locals {
      write_fmt_groups(&mut f, &create_mod_name(&l.root.key), &l.root)?;
   }
   writeln!(f, "}}")?;

   //-------------------------
   write_sep(&mut f)?;
   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn write_pre_defined_atomic_fn(r: &mut impl Write) -> anyhow::Result<()> {
   write!(
      r,
      r#"
      //----------------------------------------

      /// Atomic primitive for function pointers.
      ///
      /// [the atomic-rs crate](https://github.com/Amanieu/atomic-rs)
      /// was used as a reference.
      ///
      /// This is used to safe switching the current local functions.
      ///
      /// TODO Rewtite it as soon as Rust has an Atomic that supports function pointers.
      /// https://github.com/rust-lang/rfcs/issues/2481
      pub struct AtomicFn<T> {{
         v: core::cell::UnsafeCell<T>,
      }}
      impl<T> core::clone::Clone for AtomicFn<T> {{
         fn clone(&self) -> Self {{
            Self {{ v: core::cell::UnsafeCell::new(self.load()) }}
         }}
      }}
      impl<T> AtomicFn<T> {{
         #[inline]
         pub const fn new(v: T) -> Self {{
            type A = core::sync::atomic::AtomicUsize;
            if core::mem::size_of::<Self>() != core::mem::size_of::<A>() {{
               panic!(
                  "Type size mismatch! \
                   If you see this message then you use an unexpected/unimplemented use case. \
                   Or something was changed in the Rust std library. \
                   Or Unexpected platform."
               );
            }}
            Self {{ v: core::cell::UnsafeCell::new(v) }}
         }}
         #[inline]
         pub fn store(&self, val: T) {{
            type A = core::sync::atomic::AtomicUsize;
            unsafe {{
               (*(self.inner_ptr() as *const A))
                  .store(core::mem::transmute_copy(&val), core::sync::atomic::Ordering::Relaxed)
            }}
         }}
         #[inline]
         pub fn load(&self) -> T {{
            type A = core::sync::atomic::AtomicUsize;
            unsafe {{
               core::mem::transmute_copy(
                  &(*(self.inner_ptr() as *const A)).load(core::sync::atomic::Ordering::Relaxed),
               )
            }}
         }}
         #[inline]
         fn inner_ptr(&self) -> *mut T {{
            self.v.get() as *mut T
         }}
      }}
      unsafe impl<T: Copy + Send> Sync for AtomicFn<T> {{}}

      //----------------------------------------
      "#
   )?;
   Ok(())
}

fn write_pre_defined_traits(r: &mut impl Write) -> anyhow::Result<()> {
   write!(
      r,
      r#"
      pub trait StaticAccess {{
         fn str(&self) -> &'static str;
      }}
      "#
   )?;
   write!(
      r,
      r#"
      pub trait CowAccess {{
         fn cow(&self) -> std::borrow::Cow<'static, str>;
      }}
      "#
   )?;
   Ok(())
}

fn write_pre_defined_structs(r: &mut impl Write) -> anyhow::Result<()> {
   write!(
      r,
      r#"
      /// For 'static str
      pub struct Str(pub &'static str);

      impl StaticAccess for Str {{
         fn str(&self) -> &'static str {{
            self.0
         }}
      }}
      impl CowAccess for Str {{
         fn cow(&self) -> std::borrow::Cow<'static, str> {{
            std::borrow::Cow::Borrowed(self.0)
         }}
      }}
      impl core::fmt::Display for Str {{
         fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {{
            write!(f, "{{}}", self.0)
         }}
      }}
      "#
   )?;
   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn write_structs(r: &mut impl Write, names: &mut StructNames, item: &Item) -> anyhow::Result<()> {
   for value in &item.values {
      let args = &value.1.args;

      if args.is_empty() {
         continue;
      }

      let (created, struct_name) = names.get_or_add(args);
      if !created {
         continue;
      }

      let life_time = if value.1.has_ref() { "<'a>" } else { "" };
      //-------------------------------------------
      write!(
         r,
         r#"
         /// Arguments: `({})
         pub struct {}{} {{
            {}
            pub fmt_fn: fn(&mut core::fmt::Formatter, {})  -> core::fmt::Result,
         }}
         "#,
         seq_args("", args),
         struct_name,
         life_time,
         seq_struct_members(args),
         seq_arg_types(args),
      )?;
      //-------------------------------------------
      write!(
         r,
         r#"
         impl{} CowAccess for {}{} {{
            fn cow(&self) -> std::borrow::Cow<'static, str> {{
               std::borrow::Cow::Owned(self.to_string())
            }}
         }}
         "#,
         life_time, struct_name, life_time
      )?;
      //-------------------------------------------
      write!(
         r,
         r#"
         impl{} core::fmt::Display for {}{} {{
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {{
               (self.fmt_fn)(f, {})
            }}
         }}
         "#,
         life_time,
         struct_name,
         life_time,
         seq_arg_names("self.", args)
      )?;
   }

   for g in &item.groups {
      write_structs(r, names, g.1)?;
   }

   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Write format functions for group tree.
fn write_fmt_groups(r: &mut impl Write, mod_name: &str, item: &Item) -> anyhow::Result<()> {
   write!(r, "\npub mod {} {{", mod_name)?;

   for v in &item.values {
      write_fmt_function(r, &create_fn_name(&v.0), &v.1)?;
   }

   for item in &item.groups {
      write_fmt_groups(r, &create_mod_name(&item.0), &item.1)?;
   }

   writeln!(r, "}}")?;
   Ok(())
}

fn write_fmt_function(r: &mut impl Write, fn_name: &str, item: &ItemValue) -> anyhow::Result<()> {
   if item.args.is_empty() {
      write!(
         r,
         r#"
         pub fn {}(f: &mut core::fmt::Formatter) -> core::fmt::Result {{
            write!(f, "{}")
         }}"#,
         fn_name, item.fmt_str
      )?;
   } else {
      write!(
         r,
         r#"
         pub fn {}(f: &mut core::fmt::Formatter, {}) -> core::fmt::Result {{
            write!(f, "{}", {})
         }}"#,
         fn_name,
         seq_args("", &item.args),
         item.fmt_str,
         seq_arg_names("", &item.args),
      )?;
   }

   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Write local functions for group tree.
fn write_local_groups(
   r: &mut impl Write,
   tree_path: &mut Vec<String>,
   names: &mut StructNames,
   item: &Item,
) -> anyhow::Result<()> {
   writeln!(r, "\npub mod {} {{", tree_path.last().unwrap())?;
   writeln!(r, "     use super::*;")?;

   for v in &item.values {
      write_local_function(r, tree_path, names, &create_fn_name(&v.0), &v.1)?;
   }

   for item in &item.groups {
      tree_path.push(create_mod_name(&item.0));
      write_local_groups(r, tree_path, names, &item.1)?;
      tree_path.pop();
   }

   writeln!(r, "}}")?;
   Ok(())
}

fn write_local_function(
   r: &mut impl Write,
   tree_path: &mut Vec<String>,
   names: &mut StructNames,
   fn_name: &str,
   item: &ItemValue,
) -> anyhow::Result<()> {
   if item.args.is_empty() {
      write!(
         r,
         r#"
         /// Text: `"{}"`
         pub fn {}() -> defines::Str {{
            defines::Str("{}")
         }}"#,
         item.fmt_str, fn_name, item.fmt_str
      )?;
   } else {
      let (_, struct_name) = names.get_or_add(&item.args);
      let life_time = if item.has_ref() { "<'_>" } else { "" };

      write!(
         r,
         r#"
         /// Text: `"{}"`
         pub fn {}({}) -> defines::{}{} {{
            defines::{} {{
               {},
               fmt_fn: fmt::{}{}
            }}
         }}"#,
         item.fmt_str,
         fn_name,
         seq_args("", &item.args),
         struct_name,
         life_time,
         struct_name,
         seq_arg_names("", &item.args),
         join_tree_path(tree_path, "::"),
         fn_name
      )?;
   }

   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn write_local_statics(
   r: &mut impl Write,
   tree_path: &mut Vec<String>,
   _names: &mut StructNames,
   locals: &[Local],
) -> anyhow::Result<()> {
   let default = locals.first().unwrap();
   tree_path.push(create_mod_name(&default.root.key));
   //------------------------
   writeln!(r, "mod inner {{")?;
   //------------------------
   write_pre_defined_atomic_fn(r)?;
   //------------------------
   writeln!(r, "\n}}\n")?;
   tree_path.pop();
   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn write_local_hierarchy(
   r: &mut impl Write,
   tree_path: &mut Vec<String>,
   names: &mut StructNames,
   item: &Item,
   locals: &[Local],
) -> anyhow::Result<()> {
   let is_root_mod = tree_path.is_empty();
   let mod_name = if is_root_mod { "local".to_string() } else { create_mod_name(&item.key) };
   let local_mod_name = create_mod_name(&locals.first().unwrap().root.key);
   //////////////////////////////////////////////////////
   writeln!(r, "pub mod {} {{\n   use super::*;\n", mod_name)?;
   tree_path.push(mod_name);
   //-------------------------
   for g in &item.groups {
      write_local_hierarchy(r, tree_path, names, &g.1, locals)?;
   }
   //////////////////////////////////////////////////////
   writeln!(r, "   #[derive(Clone)]")?;
   writeln!(r, "   pub struct Local {{")?;

   for g in &item.groups {
      let group_name = create_mod_name(&g.0);
      writeln!(r, "    pub {}: {}::Local,", group_name, group_name)?;
   }

   for v in &item.values {
      let (_, struct_name) = names.get_or_add(&v.1.args);
      let life_time = if v.1.has_ref() { "<'_>" } else { "" };

      if v.1.args.is_empty() {
         writeln!(
            r,
            "    {}: inner::AtomicFn<fn({}) -> defines::Str>,",
            v.0,
            seq_arg_types(&v.1.args)
         )?;
      } else {
         writeln!(
            r,
            "    {}: inner::AtomicFn<fn({}) -> defines::{}{}>,",
            v.0,
            seq_arg_types(&v.1.args),
            struct_name,
            life_time,
         )?;
      }
   }

   writeln!(r, "   }}")?;
   //////////////////////////////////////////////////////
   writeln!(r, "   impl core::default::Default for Local {{")?;
   writeln!(r, "    fn default() -> Self {{")?;
   writeln!(r, "     Self::new_{}()", local_mod_name)?;
   writeln!(r, "    }}")?;
   writeln!(r, "   }}")?;
   //////////////////////////////////////////////////////
   writeln!(r, "   impl Local {{")?;

   for l in locals {
      let local_mod_name = create_mod_name(&l.root.key);

      writeln!(r, "    #[inline] pub const fn new_{}() -> Self {{", local_mod_name)?;
      writeln!(r, "     Self {{")?;

      for g in &item.groups {
         let group_name = create_mod_name(&g.0);
         writeln!(r, "      {}: {}::Local::new_{}(),", group_name, group_name, local_mod_name)?;
      }

      for v in &item.values {
         writeln!(
            r,
            "      {}: inner::AtomicFn::new({}::{}{}),",
            v.0,
            local_mod_name,
            join_tree_path(&tree_path[1..], "::"),
            v.0
         )?;
      }

      writeln!(r, "     }}")?;
      writeln!(r, "    }}")?;
   }

   for l in locals {
      let local_mod_name = create_mod_name(&l.root.key);

      writeln!(r, "    #[inline] pub fn set_{}(&self) {{", local_mod_name)?;

      for g in &item.groups {
         let group_name = create_mod_name(&g.0);
         writeln!(r, "      self.{}.set_{}();", group_name, local_mod_name)?;
      }

      for v in &item.values {
         writeln!(
            r,
            "      self.{}.store({}::{}{});",
            v.0,
            local_mod_name,
            join_tree_path(&tree_path[1..], "::"),
            v.0
         )?;
      }

      writeln!(r, "    }}")?;
   }
   //-------------------------
   write!(
      r,
      r#"
    /// Set the current local using key, for example: `en-EN`
    ///
    /// # Return
    ///   False if local for the specified key does not exist.
    #[inline] pub fn set(&self, key: &str) -> bool {{
       match key {{"#,
   )?;
   for l in locals {
      let moc_name = create_mod_name(&l.root.key);
      write!(
         r,
         r#"
         "{}" => {{self.set_{}(); true}},"#,
         l.root.key, moc_name
      )?;
   }
   write!(
      r,
      r#"
         _ => false,
      }}
    }}"#,
   )?;
   writeln!(r)?;
   //-------------------------
   // Functions
   for v in &item.values {
      let (_, struct_name) = names.get_or_add(&v.1.args);
      let life_time_full = if v.1.has_ref() { "<'a>" } else { "" };
      let life_time = if v.1.has_ref() { "'a" } else { "" };

      if v.1.args.is_empty() {
         writeln!(r, "    #[inline] pub fn {}(&self) -> defines::Str {{", v.0)?;
         writeln!(r, "     (self.{}.load())()", v.0)?;
         writeln!(r, "    }}")?;
      } else {
         writeln!(
            r,
            "    #[inline] pub fn {}{}(&self, {}) -> defines::{}{} {{",
            v.0,
            life_time_full,
            seq_args(life_time, &v.1.args),
            struct_name,
            life_time_full,
         )?;
         writeln!(r, "     (self.{}.load())({})", v.0, seq_arg_names("", &v.1.args))?;
         writeln!(r, "    }}")?;
      }
   }

   writeln!(r, "   }}")?;
   //-------------------------
   tree_path.pop();
   writeln!(r, "}}")?;
   Ok(())
}

fn write_global_static(r: &mut impl Write, locals: &[Local]) -> anyhow::Result<()> {
   let local_mod_name = create_mod_name(&locals.first().unwrap().root.key);
   //-------------------------
   // Global static variables and functions
   write!(
      r,
      r#"

         /// Global (static) Local instance.
         pub static GLOBAL: local::Local = local::Local::new_{}();

         /// Number of available locals.
         ///
         /// Can be used to create a simple array `[MyType; tr::local::NUMBER];`
         pub const NUMBER: usize = {};

         /// Get list of available local keys.
         pub const fn list() -> &'static[&'static str] {{
            const LIST: [&str; NUMBER] = ["#,
      local_mod_name,
      locals.len()
   )?;
   for l in locals {
      write!(
         r,
         r#"
               "{}","#,
         l.root.key
      )?;
   }
   write!(
      r,
      r#"
            ];
            &LIST
         }}
      "#,
   )?;
   //-------------------------
   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////
