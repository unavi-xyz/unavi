import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/AnimatedMorphSphere/glTF/AnimatedMorphSphere.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/AnimatedMorphSphere/glTF-Binary/AnimatedMorphSphere.glb",
  },
];

export default function AnimatedMorphSphere() {
  return <ExampleOptions options={options} />;
}
