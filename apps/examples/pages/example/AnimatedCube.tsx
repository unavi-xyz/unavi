import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/AnimatedCube/glTF/AnimatedCube.gltf",
  },
];

export default function AnimatedCube() {
  return <ExampleOptions options={options} />;
}
