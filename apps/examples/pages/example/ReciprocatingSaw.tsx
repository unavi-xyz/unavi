import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/ReciprocatingSaw/glTF/ReciprocatingSaw.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/ReciprocatingSaw/glTF-Binary/ReciprocatingSaw.glb",
  },
  {
    name: "glTF-Draco",
    href: "/models/ReciprocatingSaw/glTF-Draco/ReciprocatingSaw.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/ReciprocatingSaw/glTF-Embedded/ReciprocatingSaw.gltf",
  },
];

export default function ReciprocatingSaw() {
  return <ExampleOptions options={options} />;
}
