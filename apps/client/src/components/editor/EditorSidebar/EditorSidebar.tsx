import { useState } from "react";

import { useStore } from "../helpers/store";

import Inspect from "./Inspect/Inspect";
import Objects from "./Objects/Objects";
import Packs from "./Packs/Packs";

export default function EditorSidebar() {
  const selected = useStore((state) => state.selected);

  const [pack, setPack] = useState<string>();

  return (
    <div className="bg-white w-full h-full p-6 border-l-[1px] border-neutral-200">
      {selected ? (
        <Inspect />
      ) : pack ? (
        <Objects pack={pack} setPack={setPack} />
      ) : (
        <Packs setPack={setPack} />
      )}
    </div>
  );
}
