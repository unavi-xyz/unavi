import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/TriangleWithoutIndices/glTF/TriangleWithoutIndices.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/TriangleWithoutIndices/glTF-Embedded/TriangleWithoutIndices.gltf",
  },
];

export default function TriangleWithoutIndices() {
  return <ExampleOptions options={options} />;
}
