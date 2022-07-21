import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Triangle/glTF/Triangle.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/Triangle/glTF-Embedded/Triangle.gltf",
  },
];

export default function Triangle() {
  return <ExampleOptions options={options} />;
}
