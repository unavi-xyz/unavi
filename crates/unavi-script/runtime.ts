import { generate, GenerateOptions } from "@bytecodealliance/jco/component";

export async function build_script(bytes: Uint8Array, name: string): Promise<void> {
  console.log("Building script", name);

  let options: GenerateOptions = {
    asyncMode: { tag: "jspi", val: { imports: [], exports: [] } },
    name,
    noTypescript: true,
    strict: true,
  };

  try {
    let result = await generate(bytes, options);
    console.log("Built script", result);
  } catch (err) {
    console.error("Failed to build script", err);
  }
}
