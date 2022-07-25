use deno_core::JsRealm;

use super::*;

/// Non-`Send` resource containing the scripting runtime loaded script data
pub struct ScriptingEngine {
    /// The JavaScript runtime
    pub runtime: JsRuntime,
    /// Mapping of script handles to their evaluated data
    pub loaded_scripts: HashMap<Handle<Script>, EvaluatedScriptData>,
}

impl ScriptingEngine {
    pub fn load_script(&mut self, handle: &Handle<Script>, script: &Script) {
        // Create new JS context, isolating this script evaluation from any others
        let realm = self.runtime.create_realm().unwrap();

        // Execute the script and get the return value
        let result = realm.execute_script(self.runtime.v8_isolate(), &script.path, &script.code);

        match result {
            Ok(value) => {
                // Insert the returned value into the loaded script data map
                self.loaded_scripts
                    .insert(handle.clone_weak(), EvaluatedScriptData { value, realm });
            }
            Err(e) => {
                error!("Script Error: {}", e);
            }
        }
    }
}

/// Data for a script that has been evaluated
pub struct EvaluatedScriptData {
    pub value: v8::Global<v8::Value>,
    pub realm: JsRealm,
}

impl Default for ScriptingEngine {
    fn default() -> Self {
        // Initialize the punchy extension on deno core.
        let punchy_extension = deno_core::Extension::builder()
            // Include our JavaScript initialization code
            .js(deno_core::include_js_files!(
                prefix "punchy:ext",
                "punchy.js",
            ))
            // Add our rust operation implementations
            .ops(vec![op_log::decl()])
            .build();

        Self {
            runtime: JsRuntime::new(RuntimeOptions {
                extensions: vec![punchy_extension],
                ..default()
            }),
            loaded_scripts: HashMap::default(),
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

/// Log to the bevy log
#[deno_core::op]
fn op_log(args: OpLogArg) {
    match args.level.as_str() {
        "error" => error!(script = args.path, "{}", args.message),
        "warn" => warn!(script = args.path, "{}", args.message),
        "debug" => debug!(script = args.path, "{}", args.message),
        "trace" => trace!(script = args.path, "{}", args.message),
        // Default to info
        _ => info!(script = args.path, "{}", args.message),
    };
}
