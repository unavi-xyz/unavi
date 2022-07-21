import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/SciFiHelmet/glTF/SciFiHelmet.gltf",
  },
];

export default function SciFiHelmet() {
  return <ExampleOptions options={options} />;
}
