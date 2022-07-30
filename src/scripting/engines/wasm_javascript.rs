use bevy::prelude::*;

use crate::scripting::ScriptingEngineApi;

pub struct JavaScriptEngine;

impl FromWorld for JavaScriptEngine {
    fn from_world(world: &mut World) -> Self {
        Self
    }
}

impl ScriptingEngineApi for JavaScriptEngine {
    fn load_script(
        &self,
        handle: &bevy::prelude::Handle<crate::scripting::Script>,
        script: &crate::scripting::Script,
        reload: bool,
    ) {
        // todo!()
    }

    fn has_loaded(&self, handle: &bevy::prelude::Handle<crate::scripting::Script>) -> bool {
        // todo!()
        true
    }

    fn run_script(
        &self,
        handle: &bevy::prelude::Handle<crate::scripting::Script>,
        stage: crate::scripting::ScriptStage,
        entity_components: &mut [crate::scripting::EntityDynComponents],
    ) {
        // todo!()
    }
}
