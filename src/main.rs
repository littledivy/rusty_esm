pub(crate) mod module_loader;
pub mod runtime;

pub use crate::module_loader::EmbeddedModuleLoader;
use crate::runtime::Runtime;

use deno_core::error::AnyError;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
  let js_path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("testdata/basic.js")
    .to_string_lossy()
    .to_string();

  let mut rt = Runtime::new(js_path)?;
  let value = rt.call().await?;
  println!("{:#?}", value);
  Ok(())
}
