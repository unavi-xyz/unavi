import { useCallback, useEffect, useState } from "react";
import { FaPlay, FaStop } from "react-icons/fa";

import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { useEditorStore } from "../../store";

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
    engine.physics.send({ subject: "start", data: null });
    engine.behavior.start();

    useEditorStore.setState({ isPlaying: true });
  }, [engine, sceneLoaded]);

  const stopPlaying = useCallback(async () => {
    if (!engine || !sceneLoaded) return;

    // Exit play mode
    engine.behavior.stop();
    engine.controls = "orbit";
    engine.physics.send({ subject: "stop", data: null });

    // Reset scene
    if (scene) {
      useEditorStore.setState({ sceneLoaded: false });
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
      <IconButton onClick={isPlaying ? stopPlaying : startPlaying}>
        {isPlaying ? <FaStop className="text-sm" /> : <FaPlay className="text-sm" />}
      </IconButton>
    </Tooltip>
  );
}
