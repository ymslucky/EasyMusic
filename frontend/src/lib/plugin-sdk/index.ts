/**
 * EasyMusic Plugin SDK (frontend)
 *
 * Public API for:
 * - Plugin authors: import types from "plugin-sdk"
 * - Host app: import runtime from "plugin-sdk/runtime"
 */

export * from "./types";
export { PluginRuntime, getPluginRuntime } from "./runtime";
