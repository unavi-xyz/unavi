import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/AntiqueCamera/glTF/AntiqueCamera.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/AntiqueCamera/glTF-Binary/AntiqueCamera.glb",
  },
];

export default function AntiqueCamera() {
  return <ExampleOptions options={options} />;
}
