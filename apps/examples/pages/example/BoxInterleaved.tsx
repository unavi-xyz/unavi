import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/BoxInterleaved/glTF/BoxInterleaved.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/BoxInterleaved/glTF-Binary/BoxInterleaved.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/BoxInterleaved/glTF-Embedded/BoxInterleaved.gltf",
  },
];

export default function BoxInterleaved() {
  return <ExampleOptions options={options} />;
}
