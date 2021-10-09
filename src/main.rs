pub(crate) mod module_loader;
pub mod runtime;

pub use crate::module_loader::EmbeddedModuleLoader;
use crate::runtime::Runtime;

use deno_core::error::AnyError;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ApiResponse {
  user_id: isize,
  id: isize,
  title: String,
  completed: bool,
}

#[tokio::main]
async fn main() -> Result<(), AnyError> {
  let js_path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("testdata/basic.js")
    .to_string_lossy()
    .to_string();

  let mut rt = Runtime::new(js_path)?;
  let value: ApiResponse = rt.call(5).await?;
  println!("{:#?}", value);
  Ok(())
}
