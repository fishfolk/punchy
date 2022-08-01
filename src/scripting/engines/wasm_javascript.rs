use bevy::prelude::*;
use bevy::utils::HashMap;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_mutex::Mutex;

use crate::scripting::{EntityDynComponents, Script, ScriptStage, ScriptingEngineApi};

const LOCK_SHOULD_NOT_FAIL: &str =
    "I don't think Mutex lock should fail because we shouldn't have concurrent access";

#[wasm_bindgen]
struct Punchy;

#[wasm_bindgen]
impl Punchy {
    pub fn log(&self, message: &str, level: &str) {
        let script = current_script_path();
        match level {
            "error" => error!(script, "{}", message),
            "warn" => warn!(script, "{}", message),
            "debug" => debug!(script, "{}", message),
            "trace" => trace!(script, "{}", message),
            // Default to info
            _ => info!(script, "{}", message),
        };
    }
}

#[wasm_bindgen(module = "/src/scripting/engines/wasm_javascript/punchy.js")]
extern "C" {
    fn setup_punchy_global(punchy: Punchy);
    fn current_script_path() -> String;
}

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_name = "Object")]
    type ScriptObject;

    #[wasm_bindgen(method)]
    fn update(this: &ScriptObject);
}

pub struct JavaScriptEngine {
    scripts: Mutex<HashMap<Handle<Script>, wasm_bindgen::JsValue>>,
}

impl FromWorld for JavaScriptEngine {
    fn from_world(_: &mut World) -> Self {
        setup_punchy_global(Punchy);

        Self {
            scripts: Default::default(),
        }
    }
}

impl ScriptingEngineApi for JavaScriptEngine {
    fn load_script(&self, handle: &Handle<Script>, script: &Script, _reload: bool) {
        let function = js_sys::Function::new_no_args(&format!(
            r#"return ((Punchy) => {{
                Punchy.SCRIPT_PATH = "{path}";

                {code}

                return init();
            }})(globalThis.Punchy);"#,
            path = script.path,
            code = script
                .code
                .as_javascript()
                .expect("Only JavaScript supported")
        ));

        let output = match function.call0(&JsValue::UNDEFINED) {
            Ok(output) => output,
            Err(e) => {
                error!(%script.path, "Error executing script: {:?}", e);
                return;
            }
        };

        self.scripts
            .try_lock()
            .expect(LOCK_SHOULD_NOT_FAIL)
            .insert(handle.clone_weak(), output);
    }

    fn has_loaded(&self, handle: &Handle<Script>) -> bool {
        self.scripts
            .try_lock()
            .expect(LOCK_SHOULD_NOT_FAIL)
            .contains_key(handle)
    }

    fn run_script(
        &self,
        handle: &Handle<Script>,
        stage: ScriptStage,
        _entity_components: &mut [EntityDynComponents],
    ) {
        let try_run = || {
            let scripts = self.scripts.try_lock().expect(LOCK_SHOULD_NOT_FAIL);
            let output = scripts
                .get(handle)
                .ok_or_else(|| anyhow::format_err!("Script not loaded yet"))?;

            let output: &ScriptObject = output.dyn_ref().ok_or_else(|| {
                anyhow::format_err!(
                    "Script must have an `init` function that returns \
                    an object with an `update` function."
                )
            })?;

            match stage {
                ScriptStage::Update => output.update(),
            }

            Ok::<_, anyhow::Error>(())
        };

        if let Err(e) = try_run() {
            // TODO: add script path to error
            error!("Error running script: {}", e);
        }
    }
}
