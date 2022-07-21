import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/NormalTangentMirrorTest/glTF/NormalTangentMirrorTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/NormalTangentMirrorTest/glTF-Binary/NormalTangentMirrorTest.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/NormalTangentMirrorTest/glTF-Embedded/NormalTangentMirrorTest.gltf",
  },
];

export default function NormalTangentMirrorTest() {
  return <ExampleOptions options={options} />;
}
