use crate::model::{Item, ItemValue, Local};
use std::fs::OpenOptions;
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn generate_code(default: &Local, locals: &[Local], mod_dir: &Path) -> anyhow::Result<()> {
   let mod_file_path = mod_dir.join("tr.rs");
   let mut f =
      OpenOptions::new().read(true).write(true).truncate(true).create(true).open(mod_file_path)?;

   write_service(&mut f, &default.root, locals)?;
   write_global_functions(&mut f, &default.root)?;

   write_functions(&mut f, &default.root)?;

   for l in locals {
      write_functions(&mut f, &l.root)?;
   }

   write_fmt_functions(&mut f, &default.root)?;
   for l in locals {
      write_fmt_functions(&mut f, &l.root)?;
   }

   Ok(())
}

fn create_struct_name(s: &str) -> String {
   let mut struct_name = s.to_lowercase();
   struct_name.push_str("Display");
   struct_name
}

fn create_mod_name(s: &str) -> String {
   s.replace(" ", "_").replace("-", "_").to_lowercase()
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn write_service(r: &mut impl std::io::Write, item: &Item, locals: &[Local]) -> anyhow::Result<()> {
   writeln!(r, "#![allow(dead_code)]")?;

   writeln!(r, "pub mod service {{")?;
   writeln!(r, "   use super::*;\n")?;
   //--------------------------
   writeln!(r, "   pub struct Local {{")?;
   write_local_members(r, &item, "", "   ")?;
   writeln!(r, "   }}\n")?;

   let mod_name = &create_mod_name(&item.key);
   writeln!(r, "   impl Local {{")?;
   write_local_new_fn(r, &item, &mod_name, "", "      ")?;
   for l in locals {
      let mod_name = &create_mod_name(&l.root.key);
      write_local_new_fn(r, &l.root, &mod_name, "", "      ")?;
   }
   writeln!(r, "   }}\n\n")?;
   //--------------------------
   writeln!(r, "   pub static mut CURRENT_LOCAL: Local = Local::new_{}();\n", mod_name)?;
   writeln!(r, "   pub unsafe fn set_{}() {{", mod_name)?;
   writeln!(r, "      CURRENT_LOCAL = Local::new_{}();", mod_name)?;
   writeln!(r, "   }}\n")?;
   for l in locals {
      let mod_name = &create_mod_name(&l.root.key);
      writeln!(r, "   pub unsafe fn set_{}() {{", mod_name)?;
      writeln!(r, "      CURRENT_LOCAL = Local::new_{}();", mod_name)?;
      writeln!(r, "   }}\n")?;
   }
   //--------------------------
   writeln!(r, "   pub trait StaticStrAccess {{")?;
   writeln!(r, "      fn str(&self) -> &'static str;")?;
   writeln!(r, "   }}\n\n")?;
   //--------------------------
   writeln!(r, "   pub struct StaticStrDisplay(pub &'static str);\n")?;

   writeln!(r, "   impl StaticStrAccess for StaticStrDisplay {{")?;
   writeln!(r, "      fn str(&self) -> &'static str {{self.0}}")?;
   writeln!(r, "   }}\n")?;

   writeln!(r, "   impl std::fmt::Display for StaticStrDisplay {{")?;
   writeln!(r, "      fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{")?;
   writeln!(r, "         write!(f, \"{{}}\", self.0)")?;
   writeln!(r, "      }}")?;
   writeln!(r, "   }}\n\n")?;
   //--------------------------
   for v in &item.values {
      write_service_structs(r, &v, "   ")?;
   }

   for item in &item.groups {
      write_service_group(r, &item)?;
   }

   //--------------------------
   writeln!(r, "}}\n")?;
   //--------------------------
   Ok(())
}

fn write_local_members(
   r: &mut impl std::io::Write,
   item: &Item,
   group: &str,
   indent: &str,
) -> anyhow::Result<()> {
   for v in &item.values {
      if group.is_empty() {
         write!(r, "{}   pub {}: fn(", indent, v.0)?;
      } else {
         write!(r, "{}   pub {}_{}: fn(", indent, group, v.0)?;
      }

      for (i, a) in v.1.args.iter().enumerate() {
         if i > 0 {
            write!(r, ", {}", a.typ)?;
         } else {
            write!(r, "{}", a.typ)?;
         }
      }
      if v.1.args.is_empty() {
         writeln!(r, ") -> StaticStrDisplay,")?;
      } else {
         if group.is_empty() {
            if v.1.has_ref() {
               writeln!(r, ") -> {}<'_>,", create_struct_name(&v.0))?;
            } else {
               writeln!(r, ") -> {},", create_struct_name(&v.0))?;
            }
         } else {
            if v.1.has_ref() {
               writeln!(r, ") -> {}::{}<'_>,", group, create_struct_name(&v.0))?;
            } else {
               writeln!(r, ") -> {}::{},", group, create_struct_name(&v.0))?;
            }
         }
      }
   }

   for g in &item.groups {
      write_local_members(r, &g.1, &g.0, indent)?;
   }
   Ok(())
}

fn write_local_new_fn(
   r: &mut impl std::io::Write,
   item: &Item,
   mod_name: &str,
   group: &str,
   indent: &str,
) -> anyhow::Result<()> {
   writeln!(r, "{}pub const fn new_{}() -> Self {{", indent, mod_name)?;
   writeln!(r, "{}   Self {{", indent)?;
   write_init_local_members(r, item, group, mod_name, indent)?;
   writeln!(r, "{}   }}", indent)?;
   writeln!(r, "{}}}", indent)?;
   Ok(())
}

fn write_init_local_members(
   r: &mut impl std::io::Write,
   item: &Item,
   group: &str,
   mod_name: &str,
   indent: &str,
) -> anyhow::Result<()> {
   for v in &item.values {
      if group.is_empty() {
         writeln!(r, "{}      {}: {}::{},", indent, v.0, mod_name, v.0)?;
      } else {
         writeln!(r, "{}      {}_{}: {}::{}::{},", indent, group, v.0, mod_name, group, v.0)?;
      }
   }

   for g in &item.groups {
      write_init_local_members(r, &g.1, &g.0, mod_name, indent)?;
   }
   Ok(())
}

fn write_service_group(r: &mut impl std::io::Write, item: &(&String, &Item)) -> anyhow::Result<()> {
   writeln!(r, "   pub mod {} {{", create_mod_name(&item.0))?;

   for v in &item.1.values {
      write_service_structs(r, &v, "      ")?;
   }

   assert!(item.1.groups.is_empty(), "Unsupported depth");

   writeln!(r, "   }}\n")?;
   Ok(())
}

fn write_service_structs(
   r: &mut impl std::io::Write,
   item: &(&String, &ItemValue),
   indent: &str,
) -> anyhow::Result<()> {
   let value = &item.1;

   let life_time = if value.has_ref() { "<'a>" } else { "" };

   if !value.args.is_empty() {
      let struct_name = create_struct_name(item.0);

      writeln!(r, "{}#[allow(non_camel_case_types)]", indent)?;
      writeln!(r, "{}pub struct {}{} {{", indent, struct_name, life_time)?;

      for a in &value.args {
         if !a.has_ref() {
            writeln!(r, "{}   pub {}: {},", indent, a.name, a.typ)?;
         } else {
            writeln!(r, "{}   pub {}: &'a {},", indent, a.name, &a.typ[1..])?;
         }
      }

      write!(r, "{}   pub fmt_fn: fn(&mut std::fmt::Formatter", indent)?;
      for arg in &value.args {
         write!(r, ", {}", arg.typ)?;
      }
      writeln!(r, ") -> std::fmt::Result,")?;
      writeln!(r, "{}}}\n", indent)?;

      writeln!(
         r,
         "{}impl{} std::fmt::Display for {}{} {{",
         indent, life_time, struct_name, life_time
      )?;
      writeln!(
         r,
         "{}   fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {{",
         indent
      )?;

      write!(r, "{}      (self.fmt_fn)(f", indent)?;
      for a in &value.args {
         write!(r, ", self.{}", a.name)?;
      }
      writeln!(r, ")")?;

      writeln!(r, "{}   }}", indent)?;
      writeln!(r, "{}}}\n\n", indent)?;
   }
   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn write_global_functions(r: &mut impl std::io::Write, item: &Item) -> anyhow::Result<()> {
   for v in &item.values {
      write_global_function(r, &v, "", "")?;
   }

   for i in &item.groups {
      write_global_group(r, &i)?;
   }
   Ok(())
}

fn write_global_group(r: &mut impl std::io::Write, item: &(&String, &Item)) -> anyhow::Result<()> {
   let group_name = create_mod_name(&item.0);
   writeln!(r, "pub mod {} {{", group_name)?;
   writeln!(r, "   use super::*;\n")?;

   for v in &item.1.values {
      write_global_function(r, &v, &group_name, "   ")?;
   }

   assert!(item.1.groups.is_empty(), "Unsupported depth");

   writeln!(r, "}}\n")?;
   Ok(())
}

fn write_global_function(
   r: &mut impl std::io::Write,
   item: &(&String, &ItemValue),
   group: &str,
   indent: &str,
) -> anyhow::Result<()> {
   let fn_name = item.0.replace(" ", "_").to_lowercase();
   let value = &item.1;

   if value.args.is_empty() {
      writeln!(r, "{}pub fn {}() -> service::StaticStrDisplay {{", indent, fn_name)?;
      if group.is_empty() {
         writeln!(r, "{}   unsafe {{ (service::CURRENT_LOCAL.{})() }}", indent, item.0)?;
      } else {
         writeln!(r, "{}   unsafe {{ (service::CURRENT_LOCAL.{}_{})() }}", indent, group, item.0)?;
      }
      writeln!(r, "{}}}\n", indent)?;
   } else {
      let struct_name = if group.is_empty() {
         create_struct_name(item.0)
      } else {
         format!("{}::{}", group, create_struct_name(item.0))
      };

      write!(r, "{}pub fn {}(", indent, fn_name)?;
      for (i, arg) in value.args.iter().enumerate() {
         if i > 0 {
            write!(r, ", {}: {}", arg.name, arg.typ)?;
         } else {
            write!(r, "{}: {}", arg.name, arg.typ)?;
         }
      }

      if value.has_ref() {
         writeln!(r, ") -> service::{}<'_> {{", struct_name)?;
      } else {
         writeln!(r, ") -> service::{} {{", struct_name)?;
      }

      if group.is_empty() {
         write!(r, "{}   unsafe {{ (service::CURRENT_LOCAL.{})(", indent, item.0)?;
         for (i, arg) in value.args.iter().enumerate() {
            if i > 0 {
               write!(r, ", {}", arg.name)?;
            } else {
               write!(r, "{}", arg.name)?;
            }
         }
         writeln!(r, ") }}")?;
      } else {
         write!(r, "{}   unsafe {{ (service::CURRENT_LOCAL.{}_{})(", indent, group, item.0)?;
         for (i, arg) in value.args.iter().enumerate() {
            if i > 0 {
               write!(r, ", {}", arg.name)?;
            } else {
               write!(r, "{}", arg.name)?;
            }
         }
         writeln!(r, ") }}")?;
      }

      writeln!(r, "{}}}\n", indent)?;
   }
   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn write_functions(r: &mut impl std::io::Write, item: &Item) -> anyhow::Result<()> {
   let fmt_mod_name = format!("fmt_{}", create_mod_name(&item.key));
   writeln!(r, "pub mod {} {{", create_mod_name(&item.key))?;
   writeln!(r, "   use super::*;\n")?;

   for v in &item.values {
      write_function(r, &v, &fmt_mod_name, "", "   ")?;
   }

   for i in &item.groups {
      write_group(r, &i, &fmt_mod_name)?;
   }

   writeln!(r, "}}\n")?;
   Ok(())
}

fn write_group(
   r: &mut impl std::io::Write,
   item: &(&String, &Item),
   fmt_mod_name: &str,
) -> anyhow::Result<()> {
   let group_name = create_mod_name(&item.0);
   writeln!(r, "   pub mod {} {{", group_name)?;
   writeln!(r, "      use super::*;\n")?;

   for v in &item.1.values {
      write_function(r, &v, fmt_mod_name, &group_name, "      ")?;
   }

   assert!(item.1.groups.is_empty(), "Unsupported depth");

   writeln!(r, "   }}\n")?;
   Ok(())
}

fn write_function(
   r: &mut impl std::io::Write,
   item: &(&String, &ItemValue),
   fmt_mod_name: &str,
   group: &str,
   indent: &str,
) -> anyhow::Result<()> {
   let fn_name = item.0.replace(" ", "_").to_lowercase();
   let value = &item.1;

   if value.args.is_empty() {
      writeln!(r, "{}pub fn {}() -> service::StaticStrDisplay {{", indent, fn_name)?;
      writeln!(r, "{}   service::StaticStrDisplay(\"{}\")", indent, item.1.fmt_str)?;
      writeln!(r, "{}}}\n", indent)?;
   } else {
      let struct_name = if group.is_empty() {
         create_struct_name(item.0)
      } else {
         format!("{}::{}", group, create_struct_name(item.0))
      };

      write!(r, "{}pub fn {}(", indent, fn_name)?;
      let last = value.args.len() - 1;
      for (i, arg) in value.args.iter().enumerate() {
         if i < last {
            write!(r, "{}: {}, ", arg.name, arg.typ)?;
         } else {
            write!(r, "{}: {}", arg.name, arg.typ)?;
         }
      }

      if value.has_ref() {
         writeln!(r, ") -> service::{}<'_> {{", struct_name)?;
      } else {
         writeln!(r, ") -> service::{} {{", struct_name)?;
      }

      writeln!(r, "{}   service::{} {{", indent, struct_name)?;
      for arg in &value.args {
         writeln!(r, "{}      {},", indent, arg.name)?;
      }
      if group.is_empty() {
         writeln!(r, "{}      fmt_fn: {}::{},", indent, fmt_mod_name, item.0)?;
      } else {
         writeln!(r, "{}      fmt_fn: {}::{}::{},", indent, fmt_mod_name, group, item.0)?;
      }
      writeln!(r, "{}   }}", indent)?;

      writeln!(r, "{}}}\n", indent)?;
   }
   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn write_fmt_functions(r: &mut impl std::io::Write, item: &Item) -> anyhow::Result<()> {
   writeln!(r, "pub mod fmt_{} {{", create_mod_name(&item.key))?;

   for v in &item.values {
      write_fmt_function(r, &v, "   ")?;
   }

   for item in &item.groups {
      write_fmt_group(r, &item)?;
   }

   writeln!(r, "}}\n")?;
   Ok(())
}

fn write_fmt_group(r: &mut impl std::io::Write, item: &(&String, &Item)) -> anyhow::Result<()> {
   writeln!(r, "   pub mod {} {{", create_mod_name(&item.0))?;

   for v in &item.1.values {
      write_fmt_function(r, &v, "      ")?;
   }

   assert!(item.1.groups.is_empty(), "Unsupported depth");

   writeln!(r, "   }}\n")?;
   Ok(())
}

fn write_fmt_function(
   r: &mut impl std::io::Write,
   item: &(&String, &ItemValue),
   indent: &str,
) -> anyhow::Result<()> {
   let fn_name = item.0.replace(" ", "_").to_lowercase();
   let value = &item.1;

   if value.args.is_empty() {
      writeln!(
         r,
         "{}pub fn {}(f: &mut std::fmt::Formatter) -> std::fmt::Result {{",
         indent, fn_name
      )?;
      writeln!(r, "{}   write!(f, \"{}\")", indent, value.fmt_str)?;
   } else {
      write!(r, "{}pub fn {}(f: &mut std::fmt::Formatter", indent, fn_name)?;
      for arg in &value.args {
         write!(r, ", {}: {}", arg.name, arg.typ)?;
      }
      writeln!(r, ") -> std::fmt::Result {{")?;

      write!(r, "{}   write!(f, \"{}\"", indent, value.fmt_str)?;
      for arg in &value.args {
         write!(r, ", {}", arg.name)?;
      }
      writeln!(r, ")")?;
   }

   writeln!(r, "{}}}", indent)?;
   Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////
