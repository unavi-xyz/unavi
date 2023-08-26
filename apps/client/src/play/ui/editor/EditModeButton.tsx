import { editorStore, EngineSchedules } from "@unavi/engine";
import { useAtom } from "jotai";
import { useEffect } from "react";
import { MdConstruction } from "react-icons/md";

import { engineAtom } from "@/app/play/Client";
import { usePlayStore } from "@/app/play/playStore";
import { PlayMode } from "@/app/play/types";
import Tooltip from "@/src/ui/Tooltip";

export default function EditModeButton() {
  const mode = usePlayStore((state) => state.mode);

  const [engine] = useAtom(engineAtom);

  useEffect(() => {
    if (!engine) return;

    if (mode === PlayMode.Edit) {
      engine.queueSchedule(EngineSchedules.EnterEditMode);
    } else {
      engine.queueSchedule(EngineSchedules.ExitEditMode);
    }
  }, [mode, engine]);

  return (
    <div className="fixed bottom-0 right-0 z-20 space-x-2 p-4">
      <Tooltip text="Toggle Build Mode" side="left">
        <button
          onClick={() => {
            if (mode === PlayMode.Play) {
              editorStore.set(editorStore.enabled, true);
              usePlayStore.setState({ mode: PlayMode.Edit });
            } else {
              editorStore.set(editorStore.enabled, false);
              usePlayStore.setState({ mode: PlayMode.Play });
            }
          }}
          className={`h-[52px] w-[52px] rounded-full text-2xl backdrop-blur-lg transition active:scale-95 ${
            mode === PlayMode.Edit
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
