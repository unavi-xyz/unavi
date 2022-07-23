import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/TwoSidedPlane/glTF/TwoSidedPlane.gltf",
  },
];

export default function TwoSidedPlane() {
  return <ExampleOptions options={options} />;
}
