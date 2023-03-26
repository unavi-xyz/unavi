import { useCallback, useEffect, useState } from "react";
import { FaPlay, FaStop } from "react-icons/fa";

import { useEditorStore } from "../../../../app/editor/[id]/store";
import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";

export default function PlayButton() {
  const isPlaying = useEditorStore((state) => state.isPlaying);
  const engine = useEditorStore((state) => state.engine);
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);
  const [scene, setScene] = useState<Uint8Array>();

  const startPlaying = useCallback(async () => {
    if (!engine || !sceneLoaded) return;

    // Export scene
    useEditorStore.setState({ sceneLoaded: false });
    const glb = await engine.scene.export();
    setScene(glb);
    useEditorStore.setState({ sceneLoaded: true });

    // Enter play mode
    engine.controls = "player";
    engine.physics.send({ subject: "respawn", data: null });
    engine.render.send({ subject: "toggle_animations", data: true });
    engine.behavior.start();

    useEditorStore.setState({ isPlaying: true });
  }, [engine, sceneLoaded]);

  const stopPlaying = useCallback(async () => {
    if (!engine || !sceneLoaded) return;

    // Exit play mode
    engine.behavior.stop();
    engine.controls = "orbit";
    engine.render.send({ subject: "toggle_animations", data: false });

    // Reset scene
    if (scene) {
      useEditorStore.setState({ sceneLoaded: false, selectedId: null });
      engine.scene.clear();
      await engine.scene.addBinary(scene);
      setScene(undefined);
      useEditorStore.setState({ sceneLoaded: true });
    }

    useEditorStore.setState({ isPlaying: false });
  }, [engine, sceneLoaded, scene]);

  useEffect(() => {
    useEditorStore.setState({ stopPlaying });
  }, [stopPlaying]);

  return (
    <Tooltip text={`${isPlaying ? "Stop" : "Play"}`} side="bottom">
      <IconButton disabled={!sceneLoaded} onClick={isPlaying ? stopPlaying : startPlaying}>
        {isPlaying ? <FaStop className="text-sm" /> : <FaPlay className="text-sm" />}
      </IconButton>
    </Tooltip>
  );
}
