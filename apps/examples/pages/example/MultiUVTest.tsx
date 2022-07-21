import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/MultiUVTest/glTF/MultiUVTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/MultiUVTest/glTF-Binary/MultiUVTest.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/MultiUVTest/glTF-Embedded/MultiUVTest.gltf",
  },
];

export default function MultiUVTest() {
  return <ExampleOptions options={options} />;
}
