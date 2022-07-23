import { useState } from "react";
import { AnimationAction } from "three";

import AnimationsPage from "./AnimationsPage";
import PanelTab from "./PanelTab";
import StatsPage from "./StatsPage";

interface Props {
  animations?: AnimationAction[];
}

export default function PanelPage({ animations }: Props) {
  const hasAnimations = animations && animations.length > 0;

  const [selected, setSelected] = useState(hasAnimations ? "Animations" : "Stats");

  return (
    <div className="space-y-4">
      <div className="flex space-x-2">
        {hasAnimations && (
          <PanelTab title="Animations" selected={selected} setSelected={setSelected} />
        )}
        <PanelTab title="Stats" selected={selected} setSelected={setSelected} />
      </div>

      <div>
        {selected === "Animations" && hasAnimations && <AnimationsPage animations={animations} />}
        {selected === "Stats" && <StatsPage />}
      </div>
    </div>
  );
}
