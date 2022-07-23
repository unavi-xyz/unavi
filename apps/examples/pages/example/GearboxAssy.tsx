import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/GearboxAssy/glTF/GearboxAssy.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/GearboxAssy/glTF-Binary/GearboxAssy.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/GearboxAssy/glTF-Draco/GearboxAssy.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/GearboxAssy/glTF-Embedded/GearboxAssy.gltf",
  },
];

export default function GearboxAssy() {
  return <ExampleOptions options={options} />;
}
