import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/InterpolationTest/glTF/InterpolationTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/InterpolationTest/glTF-Binary/InterpolationTest.glb",
  },
];

export default function InterpolationTest() {
  return <ExampleOptions options={options} />;
}
