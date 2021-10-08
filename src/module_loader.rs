/// Embedded Module loader is a basically FsModuleLoader with support for loading a main
/// specifier from source.
use deno_core::error::AnyError;
use deno_core::futures::FutureExt;
use deno_core::FsModuleLoader;
use deno_core::ModuleLoader;
use deno_core::ModuleSpecifier;

use std::pin::Pin;

pub struct EmbeddedModuleLoader(pub String, pub FsModuleLoader, pub String);

impl ModuleLoader for EmbeddedModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    is_main: bool,
  ) -> Result<ModuleSpecifier, AnyError> {
    if let Ok(module_specifier) = deno_core::resolve_url(specifier) {
      if specifier == &self.2 {
        return Ok(module_specifier);
      }
    }
    self.1.resolve(specifier, referrer, is_main)
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    maybe_referrer: Option<ModuleSpecifier>,
    is_dynamic: bool,
  ) -> Pin<Box<deno_core::ModuleSourceFuture>> {
    let module_specifier = module_specifier.clone();

    if module_specifier.to_string() != self.2 {
      return self.1.load(&module_specifier, maybe_referrer, is_dynamic);
    }
    let code = self.0.to_string();
    async move {
      let specifier = module_specifier.to_string();

      return Ok(deno_core::ModuleSource {
        code,
        module_url_specified: specifier.clone(),
        module_url_found: specifier,
      });
    }
    .boxed_local()
  }
}
