import { MdConstruction } from "react-icons/md";

import { usePlayStore } from "@/app/play/store";
import { PlayMode } from "@/app/play/types";
import Tooltip from "@/src/ui/Tooltip";

export default function EditModeButton() {
  const mode = usePlayStore((state) => state.mode);

  return (
    <div className="fixed bottom-0 right-0 z-20 space-x-2 p-4">
      <Tooltip text="Toggle Build Mode" side="left">
        <button
          onClick={() => {
            if (mode === PlayMode.Play) {
              usePlayStore.setState({ mode: PlayMode.Build });
            } else {
              usePlayStore.setState({ mode: PlayMode.Play });
            }
          }}
          className={`h-[52px] w-[52px] rounded-full text-2xl backdrop-blur-lg transition active:scale-95 ${
            mode === PlayMode.Build
              ? "bg-white text-black hover:bg-white/90"
              : "bg-black/50 text-white hover:bg-black/70"
          }`}
        >
          <MdConstruction className="w-full" />
        </button>
      </Tooltip>
    </div>
  );
}
