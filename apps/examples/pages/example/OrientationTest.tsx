import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/OrientationTest/glTF/OrientationTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/OrientationTest/glTF-Binary/OrientationTest.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/OrientationTest/glTF-Embedded/OrientationTest.gltf",
  },
];

export default function OrientationTest() {
  return <ExampleOptions options={options} />;
}
