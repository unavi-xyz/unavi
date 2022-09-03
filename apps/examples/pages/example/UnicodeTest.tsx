import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Unicode❤♻Test/glTF/Unicode❤♻Test.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/Unicode❤♻Test/glTF-Binary/Unicode❤♻Test.glb",
  },
];

export default function UnicodeTest() {
  return <ExampleOptions options={options} />;
}
