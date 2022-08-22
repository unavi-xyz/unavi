import { useEffect, useState } from "react";

import { useStudioStore } from "../store";

export function useGrid() {
  const grid = useStudioStore((state) => state.grid);
  const engine = useStudioStore((state) => state.engine);

  // const [small] = useState(new GridHelper(1000, 1000, 0x666666, 0x888888));
  // const [large] = useState(new GridHelper(1000, 100, 0x666666, 0x666666));

  // // Add grid to scene
  // useEffect(() => {
  //   if (!engine) return;
  //   engine.renderManager.scene.add(small);
  //   engine.renderManager.scene.add(large);
  //   return () => {
  //     small.removeFromParent();
  //     large.removeFromParent();
  //   };
  // }, [engine, small, large]);

  // // Toggle visibility
  // useEffect(() => {
  //   small.visible = grid;
  //   large.visible = grid;
  // }, [grid, small, large]);
}
