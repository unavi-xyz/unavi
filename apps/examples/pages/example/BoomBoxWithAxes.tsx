import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/BoomBoxWithAxes/glTF/BoomBoxWithAxes.gltf",
  },
];

export default function BoomBoxWithAxes() {
  return <ExampleOptions options={options} />;
}
