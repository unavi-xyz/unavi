import { useState } from "react";

import { useStore } from "../../../helpers/editor/store";

import Inspect from "./Inspect/Inspect";
import Assets from "./Assets/Assets";
import Packs from "./Packs/Packs";

export default function EditorSidebar() {
  const selected = useStore((state) => state.selected);

  const [pack, setPack] = useState<string>();

  return (
    <div className="bg-white w-full h-full p-6 border-l-[1px] border-neutral-200">
      {selected ? (
        <Inspect />
      ) : pack ? (
        <Assets pack={pack} setPack={setPack} />
      ) : (
        <Packs setPack={setPack} />
      )}
    </div>
  );
}
