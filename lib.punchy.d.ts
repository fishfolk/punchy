// These are the TypeScript definitions for the Punchy scripting API.

/// <reference no-default-lib="true" />

/**
 * The return type of scripts must implement this interface.
 */
interface ScriptSystems {
    update(): void;
}

/**
 * The global Punchy namespace which acts as the primary interface to the game.
 */
declare namespace Punchy {
    const SCRIPT_PATH: string;
    function log(message: string, level?: string): void;
}
