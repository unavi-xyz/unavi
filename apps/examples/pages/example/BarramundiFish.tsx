import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/BarramundiFish/glTF/BarramundiFish.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/BarramundiFish/glTF-Binary/BarramundiFish.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/BarramundiFish/glTF-Draco/BarramundiFish.gltf",
  },
];

export default function BarramundiFish() {
  return <ExampleOptions options={options} />;
}
