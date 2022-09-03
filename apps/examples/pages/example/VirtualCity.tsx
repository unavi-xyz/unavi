import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/VC/glTF/VC.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/VC/glTF-Binary/VC.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/VC/glTF-Draco/VC.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/VC/glTF-Embedded/VC.gltf",
  },
];

export default function VirtualCity() {
  return <ExampleOptions options={options} />;
}
