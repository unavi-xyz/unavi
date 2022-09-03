import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/DamagedHelmet/glTF/DamagedHelmet.gltf",
  },
  {
    name: "glTF-Binary",
    href: "/models/DamagedHelmet/glTF-Binary/DamagedHelmet.glb",
  },
  {
    name: "glTF-Embedded",
    href: "/models/DamagedHelmet/glTF-Embedded/DamagedHelmet.gltf",
  },
];

export default function DamagedHelmet() {
  return <ExampleOptions options={options} />;
}
