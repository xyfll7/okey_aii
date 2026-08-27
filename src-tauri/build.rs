fn main() {
  // rust_i18n::i18n! 编译期嵌入 locales/*.yml，cargo 不会自动追踪，必须显式声明。
  println!("cargo:rerun-if-changed=locales");
  tauri_build::build()
}
