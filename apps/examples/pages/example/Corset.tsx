import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Corset/glTF/Corset.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/Corset/glTF-Binary/Corset.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/Corset/glTF-Draco/Corset.gltf",
  },
];

export default function Corset() {
  return <ExampleOptions options={options} />;
}
