import { usePlayStore } from "@/app/play/playStore";
import { LeftPanelPage } from "@/app/play/types";

import AddPage from "./Add/AddPage";
import ScenePage from "./Scene/ScenePage";

export default function Left() {
  const page = usePlayStore((state) => state.leftPage);

  return (
    <div className="fixed left-0 top-24 z-20 h-full p-4">
      <div className="h-2/3 w-72 rounded-2xl bg-neutral-900 text-white">
        {page === LeftPanelPage.Add ? (
          <AddPage />
        ) : page === LeftPanelPage.Scene ? (
          <ScenePage />
        ) : null}
      </div>
    </div>
  );
}