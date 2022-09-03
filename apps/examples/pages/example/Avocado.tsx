import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Avocado/glTF/Avocado.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/Avocado/glTF-Binary/Avocado.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/Avocado/glTF-Draco/Avocado.gltf",
  },
  {
    name: "glTF-Quantized",
    href: "/models/Avocado/glTF-Quantized/Avocado.gltf",
  },
];

export default function Avocado() {
  return <ExampleOptions options={options} />;
}
