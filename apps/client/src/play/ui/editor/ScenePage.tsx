import { usePlayStore } from "@/app/play/playStore";
import { LeftPanelPage } from "@/app/play/types";

import PanelPage from "./PanelPage";
import SceneTree from "./SceneTree";

export default function ScenePage() {
  return (
    <PanelPage title="Scene">
      <button
        onClick={() => usePlayStore.setState({ leftPage: LeftPanelPage.Add })}
        className="w-full rounded-lg bg-white/10 px-4 py-2 transition hover:bg-white/20 active:opacity-80"
      >
        Add
      </button>

      <SceneTree />
    </PanelPage>
  );
}
