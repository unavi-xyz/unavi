import { useState } from "react";

import { RenderInfo, Settings } from "../ExampleCanvas";
import AnimationsPage from "./AnimationsPage";
import PanelTab from "./PanelTab";
import SettingsPage from "./SettingsPage";
import StatsPage from "./StatsPage";

interface Props {
  animations?: any[];
  info?: RenderInfo;
  settings?: Settings;
  setSettings?: (settings: Settings) => void;
}

export default function PanelPage({ animations, info, settings, setSettings }: Props) {
  const [selected, setSelected] = useState("Stats");

  const hasAnimations = animations && animations.length > 0;

  return (
    <div className="space-y-4">
      <div className="flex space-x-2">
        {hasAnimations && (
          <PanelTab title="Animations" selected={selected} setSelected={setSelected} />
        )}
        <PanelTab title="Stats" selected={selected} setSelected={setSelected} />
        <PanelTab title="Settings" selected={selected} setSelected={setSelected} />
      </div>

      <div>
        {selected === "Animations" && hasAnimations && <AnimationsPage animations={animations} />}
        {selected === "Stats" && info && <StatsPage info={info} />}
        {selected === "Settings" && settings && setSettings && (
          <SettingsPage settings={settings} setSettings={setSettings} />
        )}
      </div>
    </div>
  );
}
