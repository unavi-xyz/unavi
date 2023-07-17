import { usePlayStore } from "@/app/play/store";
import { RightPanelPage } from "@/app/play/types";

import WorldPage from "./WorldPage";

export default function Right() {
  const page = usePlayStore((state) => state.rightPage);

  return (
    <div className="fixed right-0 top-24 z-20 h-full p-4">
      <div className="h-2/3 w-72 rounded-2xl bg-neutral-900 p-4 text-white">
        {page === RightPanelPage.World ? <WorldPage /> : null}
      </div>
    </div>
  );
}
