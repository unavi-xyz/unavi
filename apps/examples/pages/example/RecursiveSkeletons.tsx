import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/RecursiveSkeletons/glTF/RecursiveSkeletons.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/RecursiveSkeletons/glTF-Binary/RecursiveSkeletons.glb",
  },
];

export default function RecursiveSkeletons() {
  return <ExampleOptions options={options} />;
}
