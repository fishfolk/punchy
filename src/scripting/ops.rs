use bevy::{ecs::event::ManualEventReader, prelude::*, utils::HashMap};
use bevy_mod_js_scripting::{
    serde_json, JsRuntimeOp, JsScript, JsValueRef, JsValueRefs, OpContext, OpMap,
};

use crate::item::ScriptItemGrabEvent;

/// Returns the list of custom scripting ops we use for Punchy
pub fn get_ops() -> OpMap {
    let mut ops = OpMap::default();

    // Here we can insert any number custom ops that may be called from scripts. This gives us a way
    // to add any JavaScript we need to the global scope and to add Rust implementations that can be
    // called from JavaScript.
    //
    // Here `punchyGetItemGrabEvents` is the op name, which means it can be run from JavaScript by
    // calling `bevyModJsScriptingOpSync("punchyGetItemGrabEvents", argument1, anotherArgument)`;
    ops.insert("punchyGetItemGrabEvents", Box::new(ItemGetGrabEvents));

    ops
}

/// This is a helper macro that will give mutable access to a list of items in the given `TypeMap`.
///
/// This is useful when you need mutable access to multiple items in a type map. It works like a
/// [`World::resource_scope`] and removes each item from the type map, passes it into your closure
/// for modification, and then adds it back to the type map.
macro_rules! with_state {
    ($state:expr, |$($argname:ident: &mut $argty:ty),+| $body:block
    ) => {
        {
            $(
                let mut $argname = $state.remove::<$argty>().unwrap_or_default();
            )*

            let r = (|$($argname: &mut $argty),*| {
                $body
            })($(&mut $argname),*);

            $(
                $state.insert($argname);
            )*

            r
        }
    };
}

struct ItemGetGrabEvents;
impl JsRuntimeOp for ItemGetGrabEvents {
    /// Any code we return here will be added to the JavaScript runtime at initialization.
    ///
    /// In this case we make sure that there is a global `punchy` variable, and we add a function to
    /// it called `getItemGrabEvents`.
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            // Initialize Punchy global if it hasn't been created yet
            if (!globalThis.punchy) {
                globalThis.punchy = {}
            }
            
            // Add a function for getting grab events
            globalThis.punchy.getItemGrabEvents = () => {
                // Here we use the `bevyModJsScriptingOpSync` function to call our op defined in
                // Rust.
                //
                // Since the op in Rust returns an array of JsValueRefs, we have to use
                // `wrapValueRef` on each item to wrap the them in a Proxy that will make it behave
                // like a normal JavaScript object.
                return bevyModJsScriptingOpSync('punchyGetItemGrabEvents')
                    .map(x => Value.wrapValueRef(x));
            }
            "#,
        )
    }

    /// This is the Rust function that is run when JavaScript calls
    /// `bevyModJsScriptingOpSync("punchyGetItemGrabEvents")`.
    ///
    /// We can use it to do anything in the Bevy world. In this case, we read events from it and
    /// return them to JavaScript.
    fn run(
        &self,
        ctx: OpContext,
        world: &mut bevy::prelude::World,
        // Here we can read the arguments passed to the op from JavaScript, but for this op, we
        // don't need any arguments.
        _args: bevy_mod_js_scripting::serde_json::Value,
    ) -> anyhow::Result<bevy_mod_js_scripting::serde_json::Value> {
        // Get the events from the Bevy world
        let event_resource = world.get_resource::<Events<ScriptItemGrabEvent>>().unwrap();

        // Use our helper macro to access data from the `ctx.op_state`.
        //
        // `op_state` allows us to store any data that we might need to access across multiple calls
        // to our op. In this case, we need to store a HashMap of event readers.
        with_state!(
            ctx.op_state,
            |// Get a mapping of event readers, so each script will have it's own event reader.
             event_readers: &mut HashMap<
                Handle<JsScript>,
                ManualEventReader<ScriptItemGrabEvent>,
            >,
             // Get the value refs, which we can use to create values accessible in JavaScript
             value_refs: &mut JsValueRefs| {
                // Get the event reader for the script that is calling this op
                let event_reader = event_readers
                    .entry(ctx.script_info.handle.clone_weak())
                    .or_default();

                // Collect the events
                let events = event_reader
                    .iter(event_resource)
                    .cloned()
                    // Only include events that are for this script
                    .filter(|event| event.script_handle == ctx.script_info.handle)
                    // Convert the event to a JsValueRef that can be used to access the event from the script
                    .map(|event| JsValueRef::new_free(Box::new(event), value_refs))
                    .collect::<Vec<_>>();

                // Return the list of events to JS
                Ok(serde_json::to_value(events)?)
            }
        )
    }
}
