/// <reference path="../../../../assets/lib.punchy.d.ts" />

((global) => {
  // Initialize the global Punchy namespace
  global.Punchy = {};
  const core = global.Deno.core;

  // Implement the log function
  global.Punchy.log = (message, level) => {
    void core.opSync("op_log", {
      message,
      path: global.Punchy.SCRIPT_PATH,
      level,
    });
  };
})(this);
