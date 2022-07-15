mod tr;

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
   unsafe { tr::service::set_en_en() };
   println!("===============================");
   println!("Default local: 1 level");
   println!("  {}", tr::hello());
   println!("  {}", tr::greet("Man"));
   println!("  {}", tr::count(42));
   println!();

   println!("Default local: 2 level");
   println!("  {}", tr::group::hello());
   println!("  {}", tr::group::greet("Man"));
   println!("  {}", tr::group::count(42, 52));
   println!("===============================");

   unsafe { tr::service::set_ru_ru() };
   println!("===============================");
   println!("Default local: 1 level");
   println!("  {}", tr::hello());
   println!("  {}", tr::greet("Man"));
   println!("  {}", tr::count(42));
   println!();

   println!("Default local: 2 level");
   println!("  {}", tr::group::hello());
   println!("  {}", tr::group::greet("Man"));
   println!("  {}", tr::group::count(42, 52));
   println!("===============================");
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// This crate is also used for testing.
#[cfg(test)]
mod tests {
   use super::*;
   use crate::tr::service::{CowAccess, StaticStrAccess};

   #[test]
   fn test_locals() {
      unsafe { tr::service::set_en_en() };
      assert_eq!("hello world!", tr::hello().str());
      assert_eq!("hello Test!", tr::greet("Test").to_string());
      assert_eq!("number 42!", tr::count(42).to_string());

      assert_eq!("hello world from group!", tr::group::hello().cow().as_ref());
      assert_eq!("hello Test from group!", tr::group::greet("Test").to_string());
      assert_eq!("number 42 and 52 from group!", tr::group::count(42, 52).cow().as_ref());

      unsafe { tr::service::set_ru_ru() };
      assert_eq!("привет мир!", tr::hello().str());
      assert_eq!("привет Тэст!", tr::greet("Тэст").to_string());
      assert_eq!("число 42!", tr::count(42).to_string());

      assert_eq!("привет мир из группы!", tr::group::hello().cow().as_ref());
      assert_eq!("привет Тэст из группы!", tr::group::greet("Тэст").to_string());
      assert_eq!("число 42 и 52 из группы!", tr::group::count(42, 52).cow().as_ref());
   }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
