import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/SimpleMeshes/glTF/SimpleMeshes.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/SimpleMeshes/glTF-Embedded/SimpleMeshes.gltf",
  },
];

export default function SimpleMeshes() {
  return <ExampleOptions options={options} />;
}
