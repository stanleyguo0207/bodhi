use std::env;

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

  let service = LitStr::new(&service, Span::call_site());

  quote! {
    mod __bodhi_generated_config {
      include!(concat!(env!("OUT_DIR"), "/config.rs"));
    }

    pub use __bodhi_generated_config::Config;
    pub type InfraConfig = __bodhi_generated_config::infra::Config;
    pub type ServiceConfig = __bodhi_generated_config::service::Config;
    pub type ServiceConfigStore = ::bodhi_config::ConfigStore<InfraConfig, ServiceConfig>;

    fn load_service_config(profile: &str) -> ::bodhi_config::prelude::Result<Config> {
      ::bodhi_config::load_config(profile, #service)
    }

    fn load_infra_config(profile: &str) -> ::bodhi_config::prelude::Result<InfraConfig> {
      ::bodhi_config::load_layered_config(profile, #service)?.extract_infra(".")
    }

    fn load_service_layer_config(profile: &str) -> ::bodhi_config::prelude::Result<ServiceConfig> {
      ::bodhi_config::load_layered_config(profile, #service)?.extract_service(".")
    }

    fn load_service_config_store(
      profile: &str,
    ) -> ::bodhi_config::prelude::Result<ServiceConfigStore> {
      ::bodhi_config::ConfigStore::<InfraConfig, ServiceConfig>::load(profile, #service)
    }
  }
  .into()
}

fn compile_error(message: &str) -> TokenStream {
  let message = LitStr::new(message, Span::call_site());
  quote!(compile_error!(#message);).into()
}
