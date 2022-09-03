import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/MetalRoughSpheresNoTextures/glTF/MetalRoughSpheresNoTextures.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/MetalRoughSpheresNoTextures/glTF-Binary/MetalRoughSpheresNoTextures.glb",
  },
];

export default function MetalRoughSpheresNoTextures() {
  return <ExampleOptions options={options} />;
}
