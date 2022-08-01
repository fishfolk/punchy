export function setup_punchy_global(punchy) {
  globalThis.Punchy = punchy;
}
export function current_script_path() {
  return globalThis.Punchy.SCRIPT_PATH;
}
