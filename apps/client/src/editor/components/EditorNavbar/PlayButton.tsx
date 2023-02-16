import { useState } from "react";
import { FaPlay, FaStop } from "react-icons/fa";

import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { useEditorStore } from "../../store";

export default function PlayButton() {
  const isPlaying = useEditorStore((state) => state.isPlaying);
  const [scene, setScene] = useState<Uint8Array>();

  async function handlePlay() {
    const { engine } = useEditorStore.getState();
    if (!engine) return;

    if (engine.controls === "player") {
      // Exit play mode
      engine.behavior.stop();
      engine.controls = "orbit";

      engine.physics.send({ subject: "stop", data: null });

      // Reset scene
      if (scene) {
        engine.scene.clear();

        useEditorStore.setState({ sceneLoaded: false });
        await engine.scene.addBinary(scene);
        setScene(undefined);
        useEditorStore.setState({ sceneLoaded: true });
      }

      useEditorStore.setState({ isPlaying: false });
    } else {
      // Export scene
      const glb = await engine.scene.export();
      setScene(glb);

      // Enter play mode
      engine.controls = "player";
      engine.physics.send({ subject: "respawn", data: null });
      engine.physics.send({ subject: "start", data: null });
      engine.behavior.start();

      useEditorStore.setState({ isPlaying: true });
    }
  }

  return (
    <Tooltip text={`${isPlaying ? "Stop" : "Play"}`} side="bottom">
      <IconButton onClick={handlePlay}>
        {isPlaying ? <FaStop className="text-sm" /> : <FaPlay className="text-sm" />}
      </IconButton>
    </Tooltip>
  );
}
