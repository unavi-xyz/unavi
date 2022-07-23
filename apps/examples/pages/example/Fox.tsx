import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/Fox/glTF/Fox.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/Fox/glTF-Binary/Fox.glb",
  },
];

export default function Fox() {
  return <ExampleOptions options={options} />;
}
