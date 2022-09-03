import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/CesiumMan/glTF/CesiumMan.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/CesiumMan/glTF-Binary/CesiumMan.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/CesiumMan/glTF-Draco/CesiumMan.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/CesiumMan/glTF-Embedded/CesiumMan.gltf",
  },
];

export default function CesiumMan() {
  return <ExampleOptions options={options} />;
}
