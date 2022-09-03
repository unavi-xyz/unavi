import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Suzanne/glTF/Suzanne.gltf",
  },
];

export default function Suzanne() {
  return <ExampleOptions options={options} />;
}
