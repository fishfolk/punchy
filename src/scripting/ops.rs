use anyhow::Context;
use bevy::prelude::*;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, OpContext, OpMap};

pub fn get_ops() -> OpMap {
    let mut ops = OpMap::default();

    ops.insert("testOp", Box::new(TestOp));

    ops
}

struct TestOp;
impl JsRuntimeOp for TestOp {
    fn js(&self) -> Option<&'static str> {
        Some("globalThis.testOp = (...args) => bevyModJsScriptingOpSync('testOp', ...args);")
    }

    fn run(
        &self,
        _ctx: OpContext,
        _world: &mut bevy::prelude::World,
        args: bevy_mod_js_scripting::serde_json::Value,
    ) -> anyhow::Result<bevy_mod_js_scripting::serde_json::Value> {
        let (s,): (String,) = serde_json::from_value(args).context("Parse args")?;

        info!("Test op works! `{}`", s);

        Ok(serde_json::Value::Null)
    }
}
