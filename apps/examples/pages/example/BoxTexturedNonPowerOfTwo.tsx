import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/BoxTexturedNonPowerOfTwo/glTF/BoxTexturedNonPowerOfTwo.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/BoxTexturedNonPowerOfTwo/glTF-Binary/BoxTexturedNonPowerOfTwo.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/BoxTexturedNonPowerOfTwo/glTF-Embedded/BoxTexturedNonPowerOfTwo.gltf",
  },
];

export default function BoxTexturedNonPowerOfTwo() {
  return <ExampleOptions options={options} />;
}
