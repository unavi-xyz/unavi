import { EngineSchedules, useClientStore, useSceneStore } from "@unavi/engine";
import { useEffect } from "react";
import { MdConstruction } from "react-icons/md";

import { usePlayStore } from "@/app/play/playStore";
import { PlayMode } from "@/app/play/types";
import Tooltip from "@/src/ui/Tooltip";

export default function EditModeButton() {
  const mode = usePlayStore((state) => state.mode);
  const engine = useClientStore((state) => state.engine);

  useEffect(() => {
    if (!engine) return;

    if (mode === PlayMode.Edit) {
      engine.queueSchedule(EngineSchedules.EnterEditMode);
    } else {
      engine.queueSchedule(EngineSchedules.ExitEditMode);
    }
  }, [mode, engine]);

  function toggleMode() {
    if (mode === PlayMode.Play) {
      useSceneStore.setState({ enabled: true });
      usePlayStore.setState({ mode: PlayMode.Edit });
    } else {
      useSceneStore.setState({ enabled: false });
      usePlayStore.setState({ mode: PlayMode.Play });
    }
  }

  return (
    <div className="fixed bottom-0 right-0 z-20 space-x-2 p-4">
      <Tooltip text="Toggle Build Mode" side="left">
        <button
          onClick={toggleMode}
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
