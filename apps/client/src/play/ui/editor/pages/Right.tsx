import { useSceneStore } from "@unavi/engine";

import InspectPage from "./Inspect/InspectPage";
import WorldPage from "./World/WorldPage";

export default function Right() {
  const selectedId = useSceneStore((state) => state.selectedId);

  return (
    <div className="fixed right-0 top-24 z-20 h-full p-4">
      <div className="h-2/3 w-72 rounded-2xl bg-neutral-900 text-white">
        {selectedId ? <InspectPage id={selectedId} /> : <WorldPage />}
      </div>
    </div>
  );
}
