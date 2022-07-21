import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/AnimatedMorphCube/glTF/AnimatedMorphCube.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/AnimatedMorphCube/glTF-Binary/AnimatedMorphCube.glb",
  },
  {
    name: "glTF-Quantized",
    href: "/models/AnimatedMorphCube/glTF-Quantized/AnimatedMorphCube.gltf",
  },
];

export default function AnimatedMorphCube() {
  return <ExampleOptions options={options} />;
}
