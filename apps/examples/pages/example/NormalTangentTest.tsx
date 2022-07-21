import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/NormalTangentTest/glTF/NormalTangentTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/NormalTangentTest/glTF-Binary/NormalTangentTest.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/NormalTangentTest/glTF-Embedded/NormalTangentTest.gltf",
  },
];

export default function NormalTangentTest() {
  return <ExampleOptions options={options} />;
}
