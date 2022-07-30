use std::{rc::Rc, sync::Arc};

use async_channel::{Receiver, Sender};
use async_lock::RwLock;
use bevy::{prelude::*, tasks::IoTaskPool, utils::HashMap};
use dashmap::DashSet;
use deno_core::{v8, JsRuntime};

use crate::scripting::{EntityDynComponents, Script, ScriptStage, ScriptingEngineApi};

/// Non-`Send` resource containing the scripting runtime loaded script data.
///
/// Implements [`ScriptingEngineApi`].
pub struct JavaScriptEngine {
    loaded_scripts: Arc<DashSet<Handle<Script>>>,
    runtime_data: JavaScriptEngineData,
    async_request_sender: Sender<EngineLoopMessage>,
}

/// Message sent to the engine's async task loop
enum EngineLoopMessage {
    LoadScript {
        handle: Handle<Script>,
        script: Script,
        reload: bool,
    },
}

/// Engine data lock
type JavaScriptEngineData = Rc<RwLock<JavaScriptEngineInner>>;

impl ScriptingEngineApi for JavaScriptEngine {
    fn load_script(&self, handle: &Handle<Script>, script: &Script, reload: bool) {
        let already_loaded = self.loaded_scripts.contains(handle);

        if reload || !already_loaded {
            self.async_request_sender
                .try_send(EngineLoopMessage::LoadScript {
                    handle: handle.clone_weak(),
                    script: script.clone(),
                    reload,
                })
                .ok();
        }
    }

    fn has_loaded(&self, handle: &Handle<Script>) -> bool {
        self.loaded_scripts.contains(handle)
    }

    fn run_script(
        &self,
        handle: &Handle<Script>,
        stage: ScriptStage,
        entity_components: &mut [EntityDynComponents],
    ) {
        // Try to lock the engine, just skip if it can't be locked ( for instance, modules are loading )
        let mut engine = if let Some(engine) = self.runtime_data.try_write() {
            engine
        } else {
            return;
        };
        let JavaScriptEngineInner {
            scripts: modules,
            runtime,
        } = &mut *engine;

        // Try to get the loaded data for the script, skip if the script hasn't been loaded yet
        let script = if let Some(script) = modules.get(handle) {
            script
        } else {
            return;
        };

        // Get the script exports and create a new scope
        let output = &script.output;
        let scope = &mut runtime.handle_scope();
        let output = v8::Local::new(scope, output);

        // Make sure that script output was an object
        let output = if let Ok(value) = v8::Local::<v8::Object>::try_from(output) {
            value
        } else {
            warn!(%script.path, "Script default export was not an object. Skipping.");
            return;
        };

        // Figure out which function to call on the exported object
        let fn_name_str = match stage {
            ScriptStage::Update => "update",
        };
        // Get a javascript value for the name of the function to call
        let fn_name = v8::String::new_from_utf8(
            scope,
            fn_name_str.as_bytes(),
            v8::NewStringType::Internalized,
        )
        .unwrap();

        // Get the value from the object
        let script_fn = if let Some(script_fn) = output.get(scope, fn_name.into()) {
            script_fn
        } else {
            warn!(%script.path, "Script doesn't have a default export. Skipping.");
            return;
        };

        // Make sure the value is a function
        let script_fn = if let Ok(value) = v8::Local::<v8::Function>::try_from(script_fn) {
            value
        } else {
            warn!(
                %script.path,
                "Script `{}` field on default export was not a function. Skipping",
                fn_name_str
            );
            return;
        };

        // Call the function
        script_fn.call(scope, output.into(), &[]);
    }
}

/// Deno module loader implementation responsible for resolving and loading script dependencies
struct JsModuleLoader;

impl deno_core::ModuleLoader for JsModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<deno_core::ModuleSpecifier, anyhow::Error> {
        Ok(deno_core::resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        _module_specifier: &deno_core::ModuleSpecifier,
        _maybe_referrer: Option<deno_core::ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> std::pin::Pin<Box<deno_core::ModuleSourceFuture>> {
        unimplemented!("Module loading")
    }
}

pub struct JavaScriptEngineInner {
    /// The JavaScript runtime
    pub runtime: JsRuntime,
    /// Mapping of script handles to their evaluated data
    pub scripts: HashMap<Handle<Script>, EvaluatedScriptData>,
}

/// Evaluated module definition
pub struct EvaluatedScriptData {
    output: v8::Global<v8::Value>,
    path: String,
}

impl Default for JavaScriptEngineInner {
    fn default() -> Self {
        // Initialize the punchy extension on deno core.
        let punchy_extension = deno_core::Extension::builder()
            // Include our JavaScript initialization code
            .js(deno_core::include_js_files!(
                prefix "punchy:ext",
                "./native_javascript/punchy.js",
            ))
            // Add our rust operation implementations
            .ops(vec![op_log::decl()])
            .build();

        Self {
            runtime: JsRuntime::new(deno_core::RuntimeOptions {
                extensions: vec![punchy_extension],
                module_loader: Some(Rc::new(JsModuleLoader)),
                ..default()
            }),
            scripts: Default::default(),
        }
    }
}

impl FromWorld for JavaScriptEngine {
    fn from_world(world: &mut World) -> Self {
        let task_pool = world.get_resource::<IoTaskPool>().unwrap();
        let loaded_scripts = Arc::new(DashSet::default());

        let (sender, receiver) = async_channel::unbounded();
        let data = JavaScriptEngineData::default();

        // Spawn the engine task loop for handling async tasks such as module loading
        task_pool
            .spawn_local(engine_async_task_loop(
                data.clone(),
                loaded_scripts.clone(),
                receiver,
            ))
            .detach();

        Self {
            runtime_data: data,
            async_request_sender: sender,
            loaded_scripts,
        }
    }
}

/// Task spawned by the engine that handles async tasks such as script loading
///
/// Note: This async task loop was used to load scripts because initially we were using asynchronous modules
async fn engine_async_task_loop(
    data: JavaScriptEngineData,
    loaded_scripts: Arc<DashSet<Handle<Script>>>,
    receiver: Receiver<EngineLoopMessage>,
) {
    while let Ok(message) = receiver.recv().await {
        match message {
            EngineLoopMessage::LoadScript {
                handle,
                script,
                reload,
            } => {
                if loaded_scripts.contains(&handle) && !reload {
                    continue;
                }

                let mut engine = data.write().await;

                // Helper to load script
                let load_script = || {
                    // Get the script source code
                    let code = script
                        .code
                        .as_javascript()
                        .ok_or_else(|| {
                            anyhow::format_err!("Only JavaScript scripts are supported")
                        })?
                        .to_string();

                    // Append our SCRIPT_PATH variable to the module namespace
                    let code = format!(
                        r#"Punchy.SCRIPT_PATH = '{path}'; {code}"#,
                        path = script.path,
                        code = code
                    );

                    // Run the script and get it's output
                    let output = engine.runtime.execute_script(&script.path, &code)?;

                    debug!(%script.path, "Loaded script");

                    // Store the module's exported namespace in the script map
                    engine.scripts.insert(
                        handle.clone_weak(),
                        EvaluatedScriptData {
                            path: script.path,
                            output,
                        },
                    );

                    // Mark this script as loaded
                    loaded_scripts.insert(handle);

                    Ok::<_, anyhow::Error>(())
                };

                if let Err(e) = load_script() {
                    error!("Error running script: {}", e);
                }
            }
        }
    }
}

/// Arguments to [`op_log`]
#[derive(serde::Deserialize)]
pub struct OpLogArg {
    message: String,
    level: String,
    path: String,
}

/// Deno operation to log to the Bevy log
#[deno_core::op]
fn op_log(args: OpLogArg) {
    match args.level.as_str() {
        "error" => error!(from_script = args.path, "{}", args.message),
        "warn" => warn!(from_script = args.path, "{}", args.message),
        "debug" => debug!(from_script = args.path, "{}", args.message),
        "trace" => trace!(from_script = args.path, "{}", args.message),
        // Default to info
        _ => info!(from_script = args.path, "{}", args.message),
    };
}
