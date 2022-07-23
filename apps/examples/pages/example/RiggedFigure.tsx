import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/RiggedFigure/glTF/RiggedFigure.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/RiggedFigure/glTF-Binary/RiggedFigure.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/RiggedFigure/glTF-Draco/RiggedFigure.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/RiggedFigure/glTF-Embedded/RiggedFigure.gltf",
  },
];

export default function RiggedFigure() {
  return <ExampleOptions options={options} />;
}
