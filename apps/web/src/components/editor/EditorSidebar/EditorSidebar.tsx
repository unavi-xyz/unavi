import { useState } from "react";

import { useStore } from "../../../helpers/editor/store";

import Inspect from "./Inspect/Inspect";
import Pack from "./Pack/Pack";
import AssetPacks from "./AssetPacks/AssetPacks";

export default function EditorSidebar() {
  const selected = useStore((state) => state.selected);

  const [pack, setPack] = useState<string>();

  return (
    <div className="bg-white w-full h-full p-6 border-l-[1px] border-neutral-200">
      {selected ? (
        <Inspect />
      ) : pack ? (
        <Pack pack={pack} setPack={setPack} />
      ) : (
        <AssetPacks setPack={setPack} />
      )}
    </div>
  );
}
