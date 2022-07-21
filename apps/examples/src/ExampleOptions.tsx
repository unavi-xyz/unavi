import { useState } from "react";

import ExampleCanvas from "./ExampleCanvas";

export type FileOption = {
  name: string;
  href: string;
};

interface Props {
  options: FileOption[];
}

export default function ExampleOptions({ options }: Props) {
  const [src, setSrc] = useState(options[0].href);

  return (
    <>
      <div className="absolute top-0 left-0 w-full text-center z-20">
        <div className="flex justify-center mt-2">
          <div className="flex space-x-2 p-2 bg-white rounded-md">
            {options.map(({ name, href }) => {
              const selected = href === src ? "bg-gray-200" : "hover:bg-gray-200";

              return (
                <button
                  key={href}
                  onClick={() => setSrc(href)}
                  className={`px-4 py-1 transition-all rounded ${selected}`}
                >
                  {name}
                </button>
              );
            })}
          </div>
        </div>
      </div>

      <ExampleCanvas src={src} />
    </>
  );
}
