import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/SimpleMorph/glTF/SimpleMorph.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/SimpleMorph/glTF-Embedded/SimpleMorph.gltf",
  },
];

export default function SimpleMorph() {
  return <ExampleOptions options={options} />;
}
