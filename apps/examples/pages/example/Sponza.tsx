import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Sponza/glTF/Sponza.gltf",
  },
];

export default function Sponza() {
  return <ExampleOptions options={options} />;
}
