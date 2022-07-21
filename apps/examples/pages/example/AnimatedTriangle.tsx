import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/AnimatedTriangle/glTF/AnimatedTriangle.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/AnimatedTriangle/glTF-Embedded/AnimatedTriangle.gltf",
  },
];

export default function AnimatedTriangle() {
  return <ExampleOptions options={options} />;
}
