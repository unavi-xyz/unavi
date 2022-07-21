import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/SimpleSparseAccessor/glTF/SimpleSparseAccessor.gltf",
  },
  {
    name: "glTF-Embedded",
    href: "/models/SimpleSparseAccessor/glTF-Embedded/SimpleSparseAccessor.gltf",
  },
];

export default function SimpleSparseAccessor() {
  return <ExampleOptions options={options} />;
}
