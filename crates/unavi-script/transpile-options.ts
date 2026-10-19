import type { GenerateOptions } from "@bytecodealliance/jco/component";

/**
 * Every import `bindings::native`'s linker marks `async`, named the way
 * `jco`'s `asyncMode.jspi.imports` option expects: `<interface>#<name>` for a
 * free function, `<interface>#[method]<resource>.<name>` for a resource
 * method. Everything else lifts as a plain synchronous call.
 *
 * Unversioned: the import object keys `runtime.ts` returns are unversioned
 * (jco reads `imports['wired:scene/document']`), and the protocol packages
 * carry no version either, so `asyncMode`'s matcher sees one form throughout.
 */
const ASYNC_IMPORTS = [
  "wired:scene/document#open-document",
  "wired:scene/document#create-document",
  "wired:scene/document#copy-document",
  "wired:scene/document#delete-document",
  "wired:scene/document#[method]document.create-prim",
  "wired:scene/document#[method]document.apply",
  "wired:scene/document#[method]document.commit",
  "wired:scene/document#[method]document.place",
  "wired:shading/graph#set-graph",
  "wired:shading/graph#set-overrides",
  "wired:physics/simulation#raycast",
  "wired:physics/simulation#velocity",
  "wired:physics/simulation#set-velocity",
  "wired:physics/simulation#set-force",
  "wired:agent/local#attach",
  "wired:portal/portals#open",
  "wired:portal/portals#pair",
  "wired:portal/portals#travel",
  "wired:perception/perception#meta",
  "unavi:host/node-storage#get",
  "unavi:host/node-storage#list-entries",
];

/** Every export `bindings::native`'s linker marks `async`: the whole of
 * `wired:script/lifecycle`. */
const ASYNC_EXPORTS = [
  "wired:script/lifecycle#init",
  "wired:script/lifecycle#update",
  "wired:script/lifecycle#fixed-update",
];

/**
 * The worker protocol. A reply carries the `hash` of the bytes it was
 * transpiled from, which correlates it with its compile.
 */
export interface TranspileRequest {
  bytes: Uint8Array;
  hash: string;
  name: string;
}

export type TranspileReply =
  | { hash: string; js: string; modules: [string, WebAssembly.Module][] }
  | { hash: string; error: string };

/**
 * The jco options both compile paths use.
 *
 * `init`'s only WIT error is a plain `string`. `noComponentErrorWrapping`
 * keeps a failing `init` throwing that string instead of a `ComponentError`
 * wrapper, and `engine::web::tick`'s `err.as_string()` check depends on it
 * to tell a guest's own `init` failure apart from a real trap.
 */
export function transpileOptions(name: string): GenerateOptions {
  return {
    asyncMode: {
      tag: "jspi",
      val: {
        imports: ASYNC_IMPORTS,
        exports: ASYNC_EXPORTS,
      },
    },
    instantiation: { tag: "async" },
    name,
    noNamespacedExports: true,
    noNodejsCompat: true,
    noTypescript: true,
    strict: true,
    noComponentErrorWrapping: true,
  };
}
