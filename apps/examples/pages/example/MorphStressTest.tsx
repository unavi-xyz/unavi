import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/MorphStressTest/glTF/MorphStressTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/MorphStressTest/glTF-Binary/MorphStressTest.glb",
  },
];

export default function MorphStressTest() {
  return <ExampleOptions options={options} />;
}
