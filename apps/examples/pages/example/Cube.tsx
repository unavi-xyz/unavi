import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Cube/glTF/Cube.gltf",
  },
];

export default function Cube() {
  return <ExampleOptions options={options} />;
}
