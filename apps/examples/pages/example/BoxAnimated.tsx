import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/BoxAnimated/glTF/BoxAnimated.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/BoxAnimated/glTF-Binary/BoxAnimated.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/BoxAnimated/glTF-Embedded/BoxAnimated.gltf",
  },
];

export default function BoxAnimated() {
  return <ExampleOptions options={options} />;
}
