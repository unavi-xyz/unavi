import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/TextureCoordinateTest/glTF/TextureCoordinateTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/TextureCoordinateTest/glTF-Binary/TextureCoordinateTest.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/TextureCoordinateTest/glTF-Embedded/TextureCoordinateTest.gltf",
  },
];

export default function TextureCoordinateTest() {
  return <ExampleOptions options={options} />;
}
