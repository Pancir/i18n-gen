use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub fn test_gen_dir(add: Option<&str>) -> PathBuf {
   let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
   path.push("target/tests-gen");
   std::fs::create_dir_all(path.clone()).unwrap();
   if let Some(a) = add {
      path.push(a);
   }
   path
}

#[allow(dead_code)]
pub fn test_assets_dir(add: Option<&str>) -> PathBuf {
   let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
   path.push("test-assets");
   std::fs::create_dir_all(path.clone()).unwrap();
   if let Some(a) = add {
      path.push(a);
   }
   path
}

#[allow(dead_code)]
pub fn init_logger() {
   log::set_max_level(log::LevelFilter::Trace);
   if std::env::var_os("RUST_LOG").is_none() {
      std::env::set_var("RUST_LOG", "trace");
   }
   let _ = pretty_env_logger::try_init();
}

////////////////////////////////////////////////////////////////////////////////////////////////////
