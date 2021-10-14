use crate::module_loader::EmbeddedModuleLoader;
use deno_core::error::AnyError;
use deno_core::serde_json::json;
use deno_core::serde_json::Value;
use deno_core::v8;
use deno_core::FsModuleLoader;
use deno_core::ModuleSpecifier;
use deno_core::OpState;
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::ops::reg_async;
use deno_runtime::permissions::Permissions;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

fn get_error_class_name(e: &AnyError) -> &'static str {
  deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

pub struct Runtime {
  worker: MainWorker,
}

impl Runtime {
  pub async fn new(source_file: &str) -> Result<Self, AnyError> {
    let specifier = "file:///main.js".to_string();
    let source =
      format!("import * as mod from '{}';{}", source_file, BOILERPLATE);

    let module_loader = Rc::new(EmbeddedModuleLoader(
      source,
      FsModuleLoader,
      specifier.clone(),
    ));

    let create_web_worker_cb = Arc::new(|_| {
      todo!("Web workers are not supported in the example");
    });

    let options = WorkerOptions {
      apply_source_maps: false,
      args: vec![],
      cpu_count: 1,
      debug_flag: false,
      enable_testing_features: false,
      location: None,
      no_color: false,
      runtime_version: "x".to_string(),
      ts_version: "x".to_string(),
      unstable: false,
      unsafely_ignore_certificate_errors: None,
      root_cert_store: None,
      user_agent: "hello_runtime".to_string(),
      seed: None,
      js_error_create_fn: None,
      create_web_worker_cb,
      maybe_inspector_server: None,
      should_break_on_first_statement: false,
      module_loader,
      get_error_class_fn: Some(&get_error_class_name),
      origin_storage_dir: None,
      blob_store: BlobStore::default(),
      broadcast_channel: InMemoryBroadcastChannel::default(),
      shared_array_buffer_store: None,
      // compiled_wasm_module_store: None,
    };

    let main_module = deno_core::resolve_url(&specifier)?;
    let permissions = Permissions::allow_all();

    let mut worker =
      MainWorker::from_options(main_module.clone(), permissions, &options);

    worker.bootstrap(&options);
    worker.execute_main_module(&main_module).await?;

    Ok(Self { worker })
  }

  pub async fn call<R, T>(
    &mut self,
    fn_name: &str,
    arguments: &[R],
  ) -> Result<T, AnyError>
  where
    R: Serialize + 'static,
    T: Debug + DeserializeOwned + 'static,
  {
    let rt = &mut self.worker.js_runtime;

    let func = rt.execute_script("<anon>", &format!("mod.{}", fn_name))?;

    let global = {
      let scope = &mut rt.handle_scope();
      let arguments: Vec<v8::Local<v8::Value>> = arguments
        .iter()
        .map(|argument| deno_core::serde_v8::to_v8(scope, argument).unwrap())
        .collect();

      let func_obj = func.get(scope).to_object(scope).unwrap();
      let func = v8::Local::<v8::Function>::try_from(func_obj)?;

      let undefined = v8::undefined(scope);
      let local = func.call(scope, undefined.into(), &arguments).unwrap();

      v8::Global::new(scope, local)
    };

    let result: T = {
      // Run the event loop.
      let value = rt.resolve_value(global).await?;
      let scope = &mut rt.handle_scope();

      let value = v8::Local::new(scope, value);
      deno_core::serde_v8::from_v8(scope, value)?
    };

    Ok(result)
  }
}

#[cfg(feature = "op")]
const BOILERPLATE: &str = r#"
window.addEventListener("__start", async () => {
    const argument = await Deno.core.opAsync("op_recv_args");
    const result = await __fn(argument);
    Deno.core.opAsync("op_result", result);
"#;

#[cfg(feature = "rusty_v8")]
const BOILERPLATE: &str = r#"
globalThis.mod = mod;
"#;
