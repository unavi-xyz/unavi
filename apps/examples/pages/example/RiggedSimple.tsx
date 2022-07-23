import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/RiggedSimple/glTF/RiggedSimple.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/RiggedSimple/glTF-Binary/RiggedSimple.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/RiggedSimple/glTF-Draco/RiggedSimple.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/RiggedSimple/glTF-Embedded/RiggedSimple.gltf",
  },
];

export default function RiggedSimple() {
  return <ExampleOptions options={options} />;
}
