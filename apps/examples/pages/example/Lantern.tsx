import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Lantern/glTF/Lantern.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/Lantern/glTF-Binary/Lantern.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/Lantern/glTF-Draco/Lantern.gltf",
  },
  {
    name: "glTF-Quantized",
    href: "/models/Lantern/glTF-Quantized/Lantern.gltf",
  },
];

export default function Lantern() {
  return <ExampleOptions options={options} />;
}
