import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/SimpleSkin/glTF/SimpleSkin.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/SimpleSkin/glTF-Embedded/SimpleSkin.gltf",
  },
];

export default function SimpleSkin() {
  return <ExampleOptions options={options} />;
}
