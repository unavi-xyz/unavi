import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/CesiumMilkTruck/glTF/CesiumMilkTruck.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/CesiumMilkTruck/glTF-Binary/CesiumMilkTruck.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/CesiumMilkTruck/glTF-Draco/CesiumMilkTruck.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/CesiumMilkTruck/glTF-Embedded/CesiumMilkTruck.gltf",
  },
];

export default function CesiumMilkTruck() {
  return <ExampleOptions options={options} />;
}
