import { editorStore } from "@unavi/engine";
import { useAtom } from "jotai";

import InspectPage from "./Inspect/InspectPage";
import WorldPage from "./World/WorldPage";

export default function Right() {
  const [selectedId] = useAtom(editorStore.selectedId);

  return (
    <div className="fixed right-0 top-24 z-20 h-full p-4">
      <div className="h-2/3 w-80 rounded-2xl bg-neutral-900 text-white">
        {selectedId ? <InspectPage id={selectedId} /> : <WorldPage />}
      </div>
    </div>
  );
}
