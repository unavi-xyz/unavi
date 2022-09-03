import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Box/glTF/Box.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/Box/glTF-Binary/Box.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/Box/glTF-Draco/Box.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/Box/glTF-Embedded/Box.gltf",
  },
];

export default function Box() {
  return <ExampleOptions options={options} />;
}
