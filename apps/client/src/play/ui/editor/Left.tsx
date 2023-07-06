import { usePlayStore } from "@/app/play/store";
import { LeftPanelPage } from "@/app/play/types";

import AddPage from "./AddPage";

export default function Left() {
  const page = usePlayStore((state) => state.leftPage);

  return (
    <div className="fixed left-0 top-24 z-20 h-full p-4">
      <div className="h-2/3 w-72 rounded-2xl bg-neutral-900 p-4 text-white">
        {page === LeftPanelPage.Add ? <AddPage /> : null}
      </div>
    </div>
  );
}
