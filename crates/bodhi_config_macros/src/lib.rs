use std::env;
use std::path::PathBuf;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::LitStr;

#[proc_macro]
pub fn service_config(input: TokenStream) -> TokenStream {
  let input = input.to_string().trim().to_string();

  let service = if input.is_empty() {
    match env::var("CARGO_PKG_NAME") {
      Ok(value) => value,
      Err(err) => return compile_error(&format!("read CARGO_PKG_NAME failed: {err}")),
    }
  } else {
    match syn::parse_str::<LitStr>(&input) {
      Ok(value) => value.value(),
      Err(err) => {
        return compile_error(&format!(
          "service_config! expects no arguments or a single string literal: {err}"
        ));
      }
    }
  };

  let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
    Ok(value) => PathBuf::from(value),
    Err(err) => return compile_error(&format!("read CARGO_MANIFEST_DIR failed: {err}")),
  };
  let generated_path = manifest_dir.join("src").join("__bodhi_generated_config.rs");

  if !generated_path.is_file() {
    return compile_error(&format!(
      "generated config not found: {}. Run `cargo gen-config` or `cargo run -p bodhi_config -- gen-project` first.",
      generated_path.display()
    ));
  }

  let service = LitStr::new(&service, Span::call_site());
  let generated_module = LitStr::new("__bodhi_generated_config.rs", Span::call_site());

  quote! {
    #[path = #generated_module]
    mod __bodhi_generated_config;

    pub use __bodhi_generated_config::Config;
    pub type ServiceConfig = __bodhi_generated_config::Config;

    fn load_service_config(profile: &str) -> ::bodhi_config::prelude::Result<ServiceConfig> {
      ::bodhi_config::load_config(profile, #service)
    }
  }
  .into()
}

fn compile_error(message: &str) -> TokenStream {
  let message = LitStr::new(message, Span::call_site());
  quote!(compile_error!(#message);).into()
}
