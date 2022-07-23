import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/BoxVertexColors/glTF/BoxVertexColors.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/BoxVertexColors/glTF-Binary/BoxVertexColors.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/BoxVertexColors/glTF-Embedded/BoxVertexColors.gltf",
  },
];

export default function BoxVertexColors() {
  return <ExampleOptions options={options} />;
}
