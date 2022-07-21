import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/TextureSettingsTest/glTF/TextureSettingsTest.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/TextureSettingsTest/glTF-Binary/TextureSettingsTest.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/TextureSettingsTest/glTF-Embedded/TextureSettingsTest.gltf",
  },
];

export default function TextureSettingsTest() {
  return <ExampleOptions options={options} />;
}
