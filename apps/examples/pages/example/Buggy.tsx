import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Buggy/glTF/Buggy.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/Buggy/glTF-Binary/Buggy.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/Buggy/glTF-Draco/Buggy.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/Buggy/glTF-Embedded/Buggy.gltf",
  },
];

export default function Buggy() {
  return <ExampleOptions options={options} />;
}
