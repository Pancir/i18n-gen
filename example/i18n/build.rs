use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
   let i18n_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("i18n");
   let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
   i18n::generate(&i18n_dir, &out_dir).unwrap();
   println!("cargo:rerun-if-changed={}", i18n_dir.display());
}

////////////////////////////////////////////////////////////////////////////////////////////////////
