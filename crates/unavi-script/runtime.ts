import {
  generate,
  GenerateOptions,
  Transpiled,
} from "@bytecodealliance/jco/component";
import { WASIShim } from "@bytecodealliance/preview2-shim/instantiation";

export async function build_script(
  bytes: Uint8Array,
  name: string,
): Promise<void> {
  console.log("Building script", name);

  const options: GenerateOptions = {
    asyncMode: { tag: "jspi", val: { imports: [], exports: [] } },
    instantiation: { tag: "async" },
    name,
    noNodejsCompat: true,
    noTypescript: true,
    strict: true,
  };

  try {
    const result = await (generate(
      bytes,
      options,
    ) as unknown as Promise<Transpiled>);
    console.log("Generated script", name, result);

    const jsFile = result.files.find(([name]) => name.endsWith(".js"));
    if (jsFile == undefined) {
      console.warn("Transpiled JS not found");
      return;
    }
    const jsCode = new TextDecoder().decode(jsFile[1]);
    const blob = new Blob([jsCode], { type: "text/javascript" });
    const url = URL.createObjectURL(blob);

    const mod = await import(url);

    const fileMap = new Map(result.files);

    async function getCoreModule(path: string): Promise<WebAssembly.Module> {
      const bytes = fileMap.get(path);
      if (!bytes) {
        throw new Error(`Missing wasm module: ${path}`);
      }
      return await WebAssembly.compile(bytes as BufferSource);
    }

    const wasi = new WASIShim({
      sandbox: {
        preopens: {},
        env: {},
        args: [],
        enableNetwork: false,
      },
    });

    const imports = {
      ...wasi.getImportObject(),
      "wired:scene/api": {
        "self-node": () => {
          console.log("Hello from self-node");
        },
      },
    };

    // TODO create Rust api exports -> map to imports object

    const instance = await mod.instantiate(getCoreModule, imports);
    console.log("Instantiated script", name, instance);

    const script = new instance["wired:script/guest-api"].script();

    // TODO create Script class -> send to Rust -> Bevy calls tick
  } catch (err) {
    console.error("Failed to build script", err);
  }
}
