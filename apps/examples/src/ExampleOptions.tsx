import { useEffect, useState } from "react";

import ExampleCanvas from "./ExampleCanvas";
import PanelLayout from "./PanelLayout";
import { useStore } from "./store";

export type FileOption = {
  name: string;
  href: string;
};

interface Props {
  options: FileOption[];
}

export default function ExampleOptions({ options }: Props) {
  const [gltf, setGltf] = useState(options[0]?.href);

  useEffect(() => {
    useStore.setState({ uri: gltf });
  }, [gltf]);

  return (
    <>
      <PanelLayout />

      {options.length > 1 && (
        <div className="absolute top-0 left-0 z-20 w-full pl-48 text-center">
          <div className="mt-2 flex justify-center">
            <div className="flex space-x-2 rounded-full bg-white p-2">
              {options.map(({ name, href }) => {
                const selected =
                  href === gltf ? "bg-neutral-200" : "hover:bg-neutral-200";

                return (
                  <button
                    key={href}
                    onClick={() => setGltf(href)}
                    className={`rounded-full px-4 py-1 transition ${selected}`}
                  >
                    {name}
                  </button>
                );
              })}
            </div>
          </div>
        </div>
      )}

      <ExampleCanvas />
    </>
  );
}
