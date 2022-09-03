import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/MorphPrimitivesTest/glTF/MorphPrimitivesTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/MorphPrimitivesTest/glTF-Binary/MorphPrimitivesTest.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/MorphPrimitivesTest/glTF-Draco/MorphPrimitivesTest.gltf",
  },
];

export default function MorphPrimitivesTest() {
  return <ExampleOptions options={options} />;
}
