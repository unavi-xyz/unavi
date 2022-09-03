import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/BoomBox/glTF/BoomBox.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/BoomBox/glTF-Binary/BoomBox.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/BoomBox/glTF-Draco/BoomBox.gltf",
  },
];

export default function BoomBox() {
  return <ExampleOptions options={options} />;
}
