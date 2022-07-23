import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Duck/glTF/Duck.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/Duck/glTF-Binary/Duck.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/Duck/glTF-Draco/Duck.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/Duck/glTF-Embedded/Duck.gltf",
  },
  {
    name: "glTF-Quantized",
    href: "/models/Duck/glTF-Quantized/Duck.gltf",
  },
];

export default function Duck() {
  return <ExampleOptions options={options} />;
}
