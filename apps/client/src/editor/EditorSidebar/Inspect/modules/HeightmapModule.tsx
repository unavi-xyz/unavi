import { useState } from "react";
import { editorManager, sceneManager, useStore } from "../../../helpers/store";

import NumberField from "../inputs/NumberField";
import Module from "./Module";

export default function HeightmapModule() {
  const selected = useStore((state) => state.selected);
  const type = useStore((state) => state.scene.instances[selected?.id]?.type);

  const [rows, setRows] = useState(40);
  const [width, setWidth] = useState(10);

  if (!type || type !== "GLTF") return null;

  function handleHeightmapButton() {
    editorManager.generateSelectedHeightmap(width, rows);
  }

  return (
    <Module title="Heightmap">
      <div className="space-y-1">
        <div
          onClick={handleHeightmapButton}
          className="h-8 flex items-center justify-center bg-neutral-50 rounded border
                       hover:bg-neutral-100 hover:cursor-default mb-4"
        >
          Generate Heightmap
        </div>

        <NumberField
          title="Width"
          value={width}
          step={1}
          min={0}
          onChange={setWidth}
        />
        <NumberField
          title="Rows"
          value={rows}
          step={1}
          min={1}
          max={500}
          onChange={setRows}
        />
      </div>
    </Module>
  );
}
