import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/WaterBottle/glTF/WaterBottle.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/WaterBottle/glTF-Binary/WaterBottle.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/WaterBottle/glTF-Draco/WaterBottle.gltf",
  },
];

export default function WaterBottle() {
  return <ExampleOptions options={options} />;
}
