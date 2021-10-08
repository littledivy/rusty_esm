use deno_core::error::AnyError;
use deno_core::futures::FutureExt;
use deno_core::serde_json::json;
use deno_core::serde_json::Value;
use deno_core::AsyncRefCell;
use deno_core::FsModuleLoader;
use deno_core::ModuleLoader;
use deno_core::ModuleSpecifier;
use deno_core::OpState;
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::ops::{reg_async, reg_sync};
use deno_runtime::permissions::Permissions;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use std::cell::RefCell;
use std::path::Path;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

fn get_error_class_name(e: &AnyError) -> &'static str {
    deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

struct EmbeddedModuleLoader(String, FsModuleLoader, String);

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

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    // let js_path = "./export.js".to_string();
    let js_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("export.js")
        .to_string_lossy()
        .to_string();
    let specifier = "file:///main.js".to_string();

    let source = format!("import {{ verify }} from '{}';{}", js_path, BOILERPLATE,);

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

    let mut worker = MainWorker::from_options(main_module.clone(), permissions, &options);

    {
        let rt = &mut worker.js_runtime;

        async fn op_recv_args(
            state: Rc<RefCell<OpState>>,
            args: Value,
            _: (),
        ) -> Result<deno_core::serde_json::Value, AnyError> {
            let mut state = state.borrow_mut();
            let args = state.take::<Value>();
            Ok(args)
        }

        async fn op_result(
            state: Rc<RefCell<OpState>>,
            args: Value,
            _: (),
        ) -> Result<(), AnyError> {
            let mut tx = {
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
    worker.bootstrap(&options);
    worker.execute_main_module(&main_module).await?;
    let (tx, mut rx) = mpsc::channel::<Value>(16);
    {
        let rt = &mut worker.js_runtime;

        rt.execute_script("<anon>", "window.dispatchEvent(new CustomEvent('verify'))")
            .unwrap();

        rt.op_state().borrow_mut().put::<Sender<Value>>(tx.clone());
        rt.op_state().borrow_mut().put::<Value>(json!({ "rid": 0 }));
    }
    worker.run_event_loop(false).await?;

    println!("{:#?}", rx.recv().await.unwrap());
    Ok(())
}

const BOILERPLATE: &str = r#"
window.addEventListener("verify", async () => {
    const { rid } = await Deno.core.opAsync("op_recv_args");
    const result = await verify();
    Deno.core.opAsync("op_result", result);
});
"#;
