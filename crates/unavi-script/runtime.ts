import { generate, GenerateOptions } from "@bytecodealliance/jco/component";

export function buildScript(bytes: Uint8Array, name: string): void {
  let options: GenerateOptions = {
    asyncMode: { tag: "jspi", val: { imports: [], exports: [] } },
    name,
    noTypescript: true,
    strict: true,
  };
  let result = generate(bytes, options);
  console.log("Transpiled script", result);
}
