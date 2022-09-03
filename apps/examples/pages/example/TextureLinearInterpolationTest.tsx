import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/TextureLinearInterpolationTest/glTF/TextureLinearInterpolationTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/TextureLinearInterpolationTest/glTF-Binary/TextureLinearInterpolationTest.glb",
  },
];

export default function TextureLinearInterpolationTest() {
  return <ExampleOptions options={options} />;
}
