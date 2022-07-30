use std::ffi::OsStr;

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::HashMap,
};

mod engines;
pub use engines::*;

use crate::player::Player;

/// Plugin implementing the scripting API
pub struct ScriptingPlugin;

/// Type alias for our selected scripting engine. Currently we just use JavaScript/TypeScript.
pub type ScriptingEngine = engines::javascript::JavaScriptEngine;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<ScriptingEngine>()
            .add_asset_loader(ScriptAssetLoader)
            .add_asset::<Script>()
            .add_system_to_stage(CoreStage::First, load_scripts)
            .add_system_to_stage(CoreStage::Update, update_scripts);
    }
}

/// The API implemented by scripting engine implementations
pub trait ScriptingEngineApi: FromWorld {
    /// Start loading a script and return `true` if it has finished loading
    fn load_script(&self, handle: &Handle<Script>, script: &Script, reload: bool);

    /// Returns whether or not a script has been loaded yet
    fn has_loaded(&self, handle: &Handle<Script>) -> bool;

    /// Run a script
    fn run_script(
        &self,
        handle: &Handle<Script>,
        stage: ScriptStage,
        entity_components: &mut [EntityDynComponents],
    );
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum ScriptStage {
    Update,
}

/// Marker component indicating that a script has been loaded
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct LoadedScript;

struct ScriptToLoad {
    handle: Handle<Script>,
    reload: bool,
}

// System to hot reload scripts
fn load_scripts(
    mut scripts_to_load: Local<Vec<ScriptToLoad>>,
    mut events: EventReader<AssetEvent<Script>>,
    engine: NonSendMut<ScriptingEngine>,
    assets: Res<Assets<Script>>,
) {
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                scripts_to_load.push(ScriptToLoad {
                    handle: handle.clone_weak(),
                    reload: false,
                });
            }
            AssetEvent::Modified { handle } => {
                scripts_to_load.push(ScriptToLoad {
                    handle: handle.clone_weak(),
                    reload: true,
                });
            }
            _ => (),
        }
    }

    // Get the list of scripts we need to try to load
    let mut scripts = Vec::new();
    std::mem::swap(&mut *scripts_to_load, &mut scripts);

    for to_load in scripts {
        // If the script asset has loaded
        if let Some(script) = assets.get(&to_load.handle) {
            // Have the engine load the script
            engine.load_script(&to_load.handle, script, to_load.reload);

        // If the asset hasn't loaded yet
        } else {
            // Add it to the list of scripts to try to load later
            scripts_to_load.push(to_load);
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
    engine: NonSendMut<ScriptingEngine>,
    scripts: Query<&Handle<Script>>,
    mut components: Query<ScriptableComponentsQuery>,
) {
    // Collect component query into dynamic entity datas
    let mut entity_datas = components
        .iter_mut()
        .map(|x| x.get_dyn_components())
        .collect::<Vec<_>>();

    // Process each script
    for script in scripts.iter() {
        engine.run_script(script, ScriptStage::Update, &mut entity_datas);
    }
}

/// Script asset type
#[derive(TypeUuid, Clone, Debug)]
#[uuid = "d400c50b-d109-496c-8334-75bb740f5495"]
pub struct Script {
    /// The asset path the script was loaded from
    path: String,
    /// The script source code
    code: ScriptCode,
}

/// The kind of source code of a script
#[derive(Clone, Debug)]
pub enum ScriptCode {
    JavaScript(String),
}

impl ScriptCode {
    fn as_javascript(&self) -> Option<&str> {
        match self {
            ScriptCode::JavaScript(code) => Some(code),
        }
    }
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
                let parsed = deno_ast::parse_program(deno_ast::ParseParams {
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

            load_context.set_default_asset(LoadedAsset::new(Script {
                path,
                code: ScriptCode::JavaScript(code),
            }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["js", "ts"]
    }
}
