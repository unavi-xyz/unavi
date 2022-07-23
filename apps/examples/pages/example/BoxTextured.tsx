import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/BoxTextured/glTF/BoxTextured.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/BoxTextured/glTF-Binary/BoxTextured.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/BoxTextured/glTF-Embedded/BoxTextured.gltf",
  },
];

export default function BoxTextured() {
  return <ExampleOptions options={options} />;
}
