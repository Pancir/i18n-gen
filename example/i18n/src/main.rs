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

use chrono::Utc;

mod tr;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Custom struct example
pub struct DateFormatter {
   pub dt: chrono::DateTime<chrono::Utc>,
}

impl DateFormatter {
   pub fn now() -> Self {
      Self { dt: Utc::now() }
   }
}

impl core::fmt::Display for DateFormatter {
   fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
      write!(f, "{}", self.dt.format("%F (%T)"))
   }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
   let tr = &tr::GLOBAL;

   tr.set_en_en();
   println!("===============================");
   println!("No group");
   println!("  {}", tr.hello());
   println!("  {}", tr.greet("Man"));
   println!("  {}", tr.count(42));

   println!("Ggroup depth 1");
   println!("  {}", tr.group.hello());
   println!("  {}", tr.group.greet("Man"));
   println!("  {}", tr.group.count(42, 52));

   println!("Ggroup depth 2");
   println!("  {}", tr.group.group_lvl2.hello());

   tr::GLOBAL.set_ru_ru();
   println!("===============================");
   println!("No group");
   println!("  {}", tr.hello());
   println!("  {}", tr.greet("Man"));
   println!("  {}", tr.count(42));

   println!("Ggroup depth 1");
   println!("  {}", tr.group.hello());
   println!("  {}", tr.group.greet("Man"));
   println!("  {}", tr.group.count(42, 52));

   println!("Ggroup depth 2");
   println!("  {}", tr.group.group_lvl2.hello());

   println!("===============================");
   println!("Direct acces to en_en");
   println!("  {}", tr::en_en::count(42));

   println!("===============================");
   println!("Direct acces to ru_ru");
   println!("  {}", tr::ru_ru::count(42));

   println!("===============================");
   println!("Custom date time");
   println!("  {}", tr::en_en::group::group_lvl2::dt(&DateFormatter::now()));
   println!("  {}", tr::ru_ru::group::group_lvl2::dt(&DateFormatter::now()));
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// This crate is also used for testing.
#[cfg(test)]
mod tests {
   use super::*;
   use crate::tr::defines::{CowAccess, StaticAccess};

   #[test]
   fn test_locals() {
      let tr = &tr::GLOBAL;

      tr.set_en_en();
      assert_eq!("hello world!", tr.hello().str());
      assert_eq!("hello Test!", tr.greet("Test").to_string());
      assert_eq!("number 42!", tr.count(42).to_string());

      assert_eq!("hello world from group!", tr.group.hello().cow().as_ref());
      assert_eq!("hello Test from group!", tr.group.greet("Test").to_string());
      assert_eq!("number 42 and 52 from group!", tr.group.count(42, 52).cow().as_ref());

      assert_eq!("hello world from group 2!", tr.group.group_lvl2.hello().str());

      tr.set_ru_ru();
      assert_eq!("привет мир!", tr.hello().str());
      assert_eq!("привет Тэст!", tr.greet("Тэст").to_string());
      assert_eq!("число 42!", tr.count(42).to_string());

      assert_eq!("привет мир из группы!", tr.group.hello().cow().as_ref());
      assert_eq!("привет Тэст из группы!", tr.group.greet("Тэст").to_string());
      assert_eq!("число 42 и 52 из группы!", tr.group.count(42, 52).cow().as_ref());

      assert_eq!("привет мир из группы 2!", tr.group.group_lvl2.hello().str());

      assert_eq!("hello world!", tr::en_en::hello().str());
      assert_eq!("привет мир!", tr::ru_ru::hello().str());

      let dt = DateFormatter::now();
      assert_eq!(
         format!("date-time {}!", dt),
         tr::en_en::group::group_lvl2::dt(&dt).cow().as_ref()
      );
      assert_eq!(
         format!("дата-время {}!", dt),
         tr::ru_ru::group::group_lvl2::dt(&dt).cow().as_ref()
      );

      assert!(tr.set("en-EN"));
      assert_eq!("hello world!", tr.hello().str());
      assert_eq!("hello Test!", tr.greet("Test").to_string());
      assert_eq!("number 42!", tr.count(42).to_string());

      let list = tr::list();
      assert_eq!(tr::NUMBER, list.len());
      assert_eq!("en-EN", list[0]);
      assert_eq!("ru-RU", list[1]);
   }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
