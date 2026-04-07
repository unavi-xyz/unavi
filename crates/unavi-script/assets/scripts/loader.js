// Script loader for WASM component scripts.
// Rust calls these via globalThis.__unavi_loader (js_sys::Reflect).

import { transpile } from "./jco-runtime.js";

const scripts = new Map(); // scriptId → { instance }

function toUint8Array(val) {
  if (val instanceof Uint8Array) return val;
  return new Uint8Array(val);
}

export async function loadScriptBytes(scriptId, wasmBytes) {
  try {
    const result = await transpile(toUint8Array(wasmBytes), {
      name: `script_${scriptId}`,
    });

    // result.files is Array<[name, bytes]>
    const jsEntry = result.files.find(
      ([name]) => !name.includes("/") && name.endsWith(".js"),
    );
    if (!jsEntry) throw new Error("jco transpile produced no JS file");

    let src = new TextDecoder().decode(toUint8Array(jsEntry[1]));

    // Patch WASM URL references (blob URLs have no import.meta.url).
    // jco emits: new URL('./<name>', import.meta.url)
    for (const [name, bytes] of result.files) {
      if (!name.endsWith(".wasm")) continue;
      const blob = new Blob([toUint8Array(bytes)], {
        type: "application/wasm",
      });
      const wasmUrl = URL.createObjectURL(blob);
      const replacement = `new URL('${wasmUrl}')`;
      src = src
        .replaceAll(`new URL('./${name}', import.meta.url)`, replacement)
        .replaceAll(`new URL("./${name}", import.meta.url)`, replacement);
    }

    const jsBlob = new Blob([src], { type: "application/javascript" });
    const jsUrl = URL.createObjectURL(jsBlob);

    const mod = await import(jsUrl);
    URL.revokeObjectURL(jsUrl);

    globalThis.__unavi_current_script_id = scriptId;
    let instance;
    try {
      instance = new mod.guestApi.Script();
    } finally {
      globalThis.__unavi_current_script_id = null;
    }

    scripts.set(scriptId, { instance });
    console.log(`[scripts] loaded script ${scriptId}`);
    return true;
  } catch (e) {
    console.error(`[scripts] load error for script ${scriptId}:`, e);
    return false;
  }
}

export function tickScript(scriptId) {
  const script = scripts.get(scriptId);
  if (!script) return;
  globalThis.__unavi_current_script_id = scriptId;
  try {
    script.instance?.tick?.();
  } catch (e) {
    console.error(`[scripts] tick error for script ${scriptId}:`, e);
  } finally {
    globalThis.__unavi_current_script_id = null;
  }
}

export function renderScript(scriptId) {
  const script = scripts.get(scriptId);
  if (!script) return;
  globalThis.__unavi_current_script_id = scriptId;
  try {
    script.instance?.render?.();
  } catch (e) {
    console.error(`[scripts] render error for script ${scriptId}:`, e);
  } finally {
    globalThis.__unavi_current_script_id = null;
  }
}

export function unloadScript(scriptId) {
  const script = scripts.get(scriptId);
  if (script) {
    globalThis.__unavi_current_script_id = scriptId;
    try {
      script.instance?.drop?.();
    } catch (_) {
    } finally {
      globalThis.__unavi_current_script_id = null;
    }
  }
  scripts.delete(scriptId);
}

globalThis.__unavi_loader = {
  loadScriptBytes,
  renderScript,
  tickScript,
  unloadScript,
};
