import { Transpiled, generate } from "@bytecodealliance/jco/component";
import {
  transpileOptions,
  type TranspileReply,
  type TranspileRequest,
} from "./transpile-options";

/**
 * Runs the jco half of a compile off the main thread. `generate` and the
 * `WebAssembly.compile` calls happen here; the module text and core modules
 * travel back by structured clone.
 */
self.addEventListener("message", (event: MessageEvent<TranspileRequest>) => {
  void transpile(event.data);
});

async function transpile({
  bytes,
  hash,
  name,
}: TranspileRequest): Promise<void> {
  try {
    const result = await (generate(
      bytes,
      transpileOptions(name),
    ) as unknown as Promise<Transpiled>);

    const jsFile = result.files.find(([path]) => path.endsWith(".js"));
    if (jsFile == undefined) {
      throw new Error("transpiled JS not found");
    }

    const modules: [string, WebAssembly.Module][] = [];
    for (const [path, contents] of result.files) {
      if (path.endsWith(".wasm")) {
        modules.push([
          path,
          await WebAssembly.compile(contents as BufferSource),
        ]);
      }
    }

    self.postMessage({
      hash,
      js: new TextDecoder().decode(jsFile[1]),
      modules,
    } satisfies TranspileReply);
  } catch (err) {
    self.postMessage({
      hash,
      error: err instanceof Error ? err.message : String(err),
    } satisfies TranspileReply);
  }
}
