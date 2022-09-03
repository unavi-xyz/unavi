import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/AlphaBlendModeTest/glTF/AlphaBlendModeTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/AlphaBlendModeTest/glTF-Binary/AlphaBlendModeTest.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/AlphaBlendModeTest/glTF-Embedded/AlphaBlendModeTest.gltf",
  },
];

export default function AlphaBlendModeTest() {
  return <ExampleOptions options={options} />;
}
