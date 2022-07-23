import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/2CylinderEngine/glTF/2CylinderEngine.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/2CylinderEngine/glTF-Binary/2CylinderEngine.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/2CylinderEngine/glTF-Draco/2CylinderEngine.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/2CylinderEngine/glTF-Embedded/2CylinderEngine.gltf",
  },
];

export default function TwoCylinderEngine() {
  return <ExampleOptions options={options} />;
}
