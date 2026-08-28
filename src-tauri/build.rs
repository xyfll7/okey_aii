fn main() {
  // rust_i18n::i18n! embeds locales/*.yml at compile time; cargo doesn't track it automatically, so it must be declared explicitly.
  println!("cargo:rerun-if-changed=locales");
  tauri_build::build()
}
