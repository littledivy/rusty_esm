#![feature(test)]
extern crate test;

pub(crate) mod module_loader;
pub mod runtime;

pub use crate::module_loader::EmbeddedModuleLoader;
use crate::runtime::Runtime;

use deno_core::error::AnyError;
use deno_core::serde_json::Value;
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
    .join("examples/async.js")
    .to_string_lossy()
    .to_string();

  let mut rt = Runtime::new(&js_path).await?;
  let value: [ApiResponse; 2] = rt.call("hello", &[5, 4]).await?;

  println!("Basic: {:#?}", value);

  let js_path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("examples/multi_type.js")
    .to_string_lossy()
    .to_string();

  let mut rt = Runtime::new(&js_path).await?;

  // Multi typed argument and return values.
  let value: Vec<Value> = rt
    .call(
      "handler",
      &[
        Value::String("Hello there".to_string()),
        Value::Number(100.into()),
      ],
    )
    .await?;

  println!("Heterogeneous: {:#?}", value);

  let js_path = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("examples/multi_export.js")
    .to_string_lossy()
    .to_string();

  let mut rt = Runtime::new(&js_path).await?;
  let sum: i32 = rt.call("sum", &[4, 5]).await?;
  let product: i32 = rt.call("product", &[4, 5]).await?;
  println!(
    "Multi export: sum of 4 and 5 is: {}, and product is: {}",
    sum, product
  );

  Ok(())
}

#[cfg(test)]
mod tests {

  use super::*;
  use test::Bencher;

  #[bench]
  fn bench_call(b: &mut Bencher) {
    let js_path = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("examples/multi_export.js")
      .to_string_lossy()
      .to_string();
    let tokio_rt = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap();
    let mut rt = tokio_rt.block_on(Runtime::new(&js_path)).unwrap();

    b.iter(|| {
      tokio_rt
        .block_on(rt.call::<_, usize>("sum", &[1, 2]))
        .unwrap();
    });
  }

  #[bench]
  fn bench_call_promise(b: &mut Bencher) {
    let js_path = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("examples/async.js")
      .to_string_lossy()
      .to_string();
    let tokio_rt = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap();
    let mut rt = tokio_rt.block_on(Runtime::new(&js_path)).unwrap();
    b.iter(|| {
      tokio_rt
        .block_on(rt.call::<(), bool>("bench", &[]))
        .unwrap();
    });
  }
}
