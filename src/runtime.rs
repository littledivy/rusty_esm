use crate::module_loader::EmbeddedModuleLoader;
use deno_core::error::AnyError;
use deno_core::serde_json::json;
use deno_core::serde_json::Value;
use deno_core::FsModuleLoader;
use deno_core::ModuleSpecifier;
use deno_core::OpState;
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::ops::reg_async;
use deno_runtime::permissions::Permissions;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

fn get_error_class_name(e: &AnyError) -> &'static str {
  deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

pub struct Runtime {
  worker: MainWorker,
  options: WorkerOptions,
  module: ModuleSpecifier,
}

impl Runtime {
  pub fn new(source_file: String) -> Result<Self, AnyError> {
    let specifier = "file:///main.js".to_string();

    let source =
      format!("import {{ verify }} from '{}';{}", source_file, BOILERPLATE);

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

    let worker =
      MainWorker::from_options(main_module.clone(), permissions, &options);

    Ok(Self {
      worker,
      options,
      module: main_module,
    })
  }

  pub async fn call(&mut self) -> Result<Value, AnyError> {
    {
      let rt = &mut self.worker.js_runtime;

      async fn op_recv_args(
        state: Rc<RefCell<OpState>>,
        _args: (),
        _: (),
      ) -> Result<Value, AnyError> {
        let mut state = state.borrow_mut();
        let args = state.take::<Value>();
        Ok(args)
      }

      async fn op_result(
        state: Rc<RefCell<OpState>>,
        args: Value,
        _: (),
      ) -> Result<(), AnyError> {
        let tx = {
          let mut state = state.borrow_mut();
          let tx = state.take::<Sender<Value>>();

          tx
        };
        tx.send(args).await.unwrap();
        Ok(())
      }

      reg_async(rt, "op_recv_args", op_recv_args);
      reg_async(rt, "op_result", op_result);
      rt.sync_ops_cache();
    }

    self.worker.bootstrap(&self.options);
    self.worker.execute_main_module(&self.module).await?;

    let (tx, mut rx) = mpsc::channel::<Value>(16);

    {
      let rt = &mut self.worker.js_runtime;

      rt.execute_script(
        "<anon>",
        "window.dispatchEvent(new CustomEvent('verify'))",
      )
      .unwrap();

      rt.op_state().borrow_mut().put::<Sender<Value>>(tx.clone());
      rt.op_state().borrow_mut().put::<Value>(json!({ "rid": 0 }));
    }

    self.worker.run_event_loop(false).await?;
    let result = rx.recv().await.unwrap();

    Ok(result)
  }
}

const BOILERPLATE: &str = r#"
window.addEventListener("verify", async () => {
    const { rid } = await Deno.core.opAsync("op_recv_args");
    const result = await verify();
    Deno.core.opAsync("op_result", result);
});
"#;
