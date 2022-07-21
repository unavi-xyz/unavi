import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/VertexColorTest/glTF/VertexColorTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/VertexColorTest/glTF-Binary/VertexColorTest.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/VertexColorTest/glTF-Embedded/VertexColorTest.gltf",
  },
];

export default function VertexColorTest() {
  return <ExampleOptions options={options} />;
}
