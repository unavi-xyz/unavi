import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/BrainStem/glTF/BrainStem.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/BrainStem/glTF-Binary/BrainStem.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/BrainStem/glTF-Draco/BrainStem.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/BrainStem/glTF-Embedded/BrainStem.gltf",
  },
];

export default function BrainStem() {
  return <ExampleOptions options={options} />;
}
