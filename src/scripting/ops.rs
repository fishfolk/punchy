use std::{cell::RefCell, rc::Rc};

use bevy::{ecs::event::ManualEventReader, prelude::*, utils::HashMap};
use bevy_mod_js_scripting::{
    bevy_ecs_dynamic::reflect_value_ref::ReflectValueRef, serde_json, JsRuntimeOp, JsScript,
    JsValueRef, JsValueRefs, OpContext, OpMap,
};

use crate::item::ScriptItemGrabEvent;

pub fn get_ops() -> OpMap {
    let mut ops = OpMap::default();

    ops.insert("punchyGetItemGrabEvents", Box::new(ItemGetGrabEvents));

    ops
}

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
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.punchy) {
                globalThis.punchy = {}
            }
            globalThis.punchy.getItemGrabEvents = () => {
                return bevyModJsScriptingOpSync('punchyGetItemGrabEvents')
                    .map(x => globalThis.wrapValueRef(x));
            }
            "#,
        )
    }

    fn run(
        &self,
        ctx: OpContext,
        world: &mut bevy::prelude::World,
        _args: bevy_mod_js_scripting::serde_json::Value,
    ) -> anyhow::Result<bevy_mod_js_scripting::serde_json::Value> {
        let event_resource = world.get_resource::<Events<ScriptItemGrabEvent>>().unwrap();

        with_state!(
            ctx.op_state,
            |event_readers: &mut HashMap<
                Handle<JsScript>,
                ManualEventReader<ScriptItemGrabEvent>,
            >,
             value_refs: &mut JsValueRefs| {
                let event_reader = event_readers
                    .entry(ctx.script_info.handle.clone_weak())
                    .or_default();

                let events = event_reader
                    .iter(event_resource)
                    .cloned()
                    .filter(|x| x.script_handle == ctx.script_info.handle)
                    .map(|x| {
                        let reflect: Box<dyn Reflect> = Box::new(x);
                        let refcell = Rc::new(RefCell::new(reflect));
                        let reflect_ref = ReflectValueRef::free(refcell);

                        JsValueRef {
                            key: value_refs.insert(reflect_ref),
                            function: None,
                        }
                    })
                    .collect::<Vec<_>>();

                Ok(serde_json::to_value(&events)?)
            }
        )
    }
}
