export function setup_js_globals(bevyModJsScripting, op_name_map_str) {
    const op_name_map = JSON.parse(op_name_map_str);

    // Set the bevy scripting op function to Deno's opSync function
    window.bevyModJsScriptingOpSync = (op_name, ...args) => {
        try {
            return bevyModJsScripting.op_sync(op_name_map[op_name], args);
        } catch(e) {
            throw `Error trying to run op \`${op_name}\`: ${e}`
        }
    }
}
