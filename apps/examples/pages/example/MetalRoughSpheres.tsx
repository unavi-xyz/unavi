import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/MetalRoughSpheres/glTF/MetalRoughSpheres.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/MetalRoughSpheres/glTF-Binary/MetalRoughSpheres.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/MetalRoughSpheres/glTF-Embedded/MetalRoughSpheres.gltf",
  },
];

export default function MetalRoughSpheres() {
  return <ExampleOptions options={options} />;
}
