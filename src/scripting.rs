use std::ffi::OsStr;

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
};
use deno_core::{v8, JsRuntime, RuntimeOptions};

mod engine;
pub use engine::*;

use crate::{config::EngineConfig, player::Player, GameStage};

/// Plugin implementing the scripting API
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<ScriptingEngine>()
            .add_asset_loader(ScriptAssetLoader)
            .add_asset::<Script>()
            .add_system_to_stage(CoreStage::First, load_scripts)
            .add_system_to_stage(CoreStage::Update, update_scripts);

        // Configure hot reload if enabled
        let engine_config = app.world.get_resource::<EngineConfig>().unwrap();
        if engine_config.hot_reload {
            app.add_system_to_stage(GameStage::HotReload, hot_reload_scripts);
        }
    }
}

/// Marker component indicating that a script has been loaded
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct LoadedScript;

/// System to load scripts that have not been evaluated yet
fn load_scripts(
    mut commands: Commands,
    mut engine: NonSendMut<ScriptingEngine>,
    unloaded_scripts: Query<(Entity, &Handle<Script>), Without<LoadedScript>>,
    script_assets: Res<Assets<Script>>,
) {
    // For each script
    for (entity, handle) in unloaded_scripts.iter() {
        // See if the asset has loaded
        if let Some(script) = script_assets.get(handle) {
            engine.load_script(handle, script);

            // Add marker indicating the script has been loaded
            commands.entity(entity).insert(LoadedScript);
        }
    }
}

// System to hot reload scripts
fn hot_reload_scripts(
    mut events: EventReader<AssetEvent<Script>>,
    mut engine: NonSendMut<ScriptingEngine>,
    assets: Res<Assets<Script>>,
) {
    for event in events.iter() {
        if let AssetEvent::Modified { handle } = event {
            let script = assets.get(handle).unwrap();

            engine.load_script(handle, script);
        }
    }
}

/// Macro to define a query type for all of our scriptable components
macro_rules! scriptable_components {
    ( $( $component:ident ),+ $(,)? ) => {
        use scriptable_components::*;
        // Put in it's own module so we can scope the `allow(non_snake_case)`
        mod scriptable_components {
            #![allow(non_snake_case)]
            use super::*;

            // Assert components impl reflect
            trait AssertReflect: Reflect {}
            $(
                impl AssertReflect for $component {}
            )+

            #[derive(::bevy::ecs::query::WorldQuery)]
            #[world_query(mutable)]
            pub struct ScriptableComponentsQuery<'w> {
                entity: Entity,
                $(
                    $component: Option<&'w mut $component>,
                )+
            }

            impl<'w> ScriptableComponentsQueryItem<'w> {
                pub fn get_dyn_components(self) -> EntityDynComponents<'w> {
                    let mut components = HashMap::default();

                    $(
                        if let Some(component) = self.$component {
                            let name = component.type_name();
                            components.insert(name.into(), component.into_inner() as &mut dyn Reflect);
                        }
                    )+

                    EntityDynComponents {
                        entity: self.entity,
                        components,
                    }
                }
            }
        }
    };
}

// Here we list all of our scriptable components
scriptable_components!(Transform, Player);

/// A struct that contains an entity ID and all of it's scriptable components as a hash map of the
/// component's type ID and a mutable reference to it as a [`&mut dyn Reflect`].
#[allow(dead_code)]
pub struct EntityDynComponents<'a> {
    entity: Entity,
    components: HashMap<String, &'a mut dyn Reflect>,
}

/// System to run scripts for [`CoreStage::Update`]
pub fn update_scripts(
    mut engine: NonSendMut<ScriptingEngine>,
    scripts: Query<&Handle<Script>, With<LoadedScript>>,
    mut components: Query<ScriptableComponentsQuery>,
) {
    let ScriptingEngine {
        runtime,
        loaded_scripts,
    } = &mut *engine;

    // Collect component query into dynamic entity datas
    let _entity_datas = components
        .iter_mut()
        .map(|x| x.get_dyn_components())
        .collect::<Vec<_>>();

    // TODO: Convert entity datas to JS args that we can pass to scripts

    // Process each loaded script
    for script in scripts.iter() {
        // Get the return value of the script and establish a local scope
        let isolate = runtime.v8_isolate();
        let EvaluatedScriptData { realm, value } = loaded_scripts.get(script).unwrap();
        let value = value.open(isolate);
        let mut scope = realm.handle_scope(isolate);

        // If the return value was an object
        if let Some(object) = value.to_object(&mut scope) {
            // Get the name of the update function
            let update_fn_name =
                v8::String::new_from_utf8(&mut scope, b"update", v8::NewStringType::Internalized)
                    .unwrap();

            // If the update property exists on the object
            if let Some(update_fn) = object.get(&mut scope, update_fn_name.into()) {
                // If the property is a function
                if update_fn.is_function() {
                    // SAFE: We check that this is a function before casting it
                    let update_fn = unsafe { v8::Local::<v8::Function>::cast(update_fn) };

                    update_fn.call(&mut scope, object.into(), &[]);
                }
            }
        }
    }
}

/// Script asset type
#[derive(TypeUuid)]
#[uuid = "d400c50b-d109-496c-8334-75bb740f5495"]
pub struct Script {
    /// The asset path the script was loaded from
    path: String,
    /// The script source code
    code: String,
}

/// Asset loader for [`Script`]
struct ScriptAssetLoader;

impl AssetLoader for ScriptAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            // Check whether or not the asset is a typescript file
            let is_typescript = load_context.path().extension() == Some(OsStr::new("ts"));

            // Get the source string
            let source = String::from_utf8(bytes.to_vec())?;

            // Transpile the source code to plain JavaScript if it's a typescript file
            let code = if is_typescript {
                let parsed = deno_ast::parse_module(deno_ast::ParseParams {
                    specifier: "source".into(),
                    text_info: deno_ast::SourceTextInfo::from_string(source),
                    media_type: deno_ast::MediaType::TypeScript,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })?;

                parsed
                    .transpile(&deno_ast::EmitOptions {
                        inline_source_map: false,
                        ..Default::default()
                    })?
                    .text
            } else {
                source
            };

            // Get the script path
            let path = load_context.path().to_string_lossy().into();

            // Prepend the SCRIPT_PATH global to the script source
            let code = format!(
                "Punchy.SCRIPT_PATH = '{path}'\n{code}",
                path = path,
                code = code
            );

            load_context.set_default_asset(LoadedAsset::new(Script { path, code }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["js", "ts"]
    }
}
