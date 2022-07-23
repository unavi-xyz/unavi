import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Box%20With%20Spaces/glTF/Box%20With%20Spaces.gltf",
  },
];

export default function BoxWithSpaces() {
  return <ExampleOptions options={options} />;
}
