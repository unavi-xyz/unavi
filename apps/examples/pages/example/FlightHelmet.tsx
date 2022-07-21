import ExampleOptions, { FileOption } from "../../src/ExampleOptions";

const options: FileOption[] = [
  {
    name: "glTF",
    href: "/models/FlightHelmet/glTF/FlightHelmet.gltf",
  },
];

export default function FlightHelmet() {
  return <ExampleOptions options={options} />;
}
