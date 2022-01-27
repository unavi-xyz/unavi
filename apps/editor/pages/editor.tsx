import { useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import { Debug, Physics } from "@react-three/cannon";
import { IconButton } from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import Split from "react-split";
import { RAYCASTER_SETTINGS } from "3d";

import { useStore } from "../src/state/useStore";
import { useScene } from "../src/state/useScene";
import { useHotkeys } from "../src/hooks/useHotkeys";

import Navbar from "../src/ui/navbar/Navbar";
import RightPanel from "../src/ui/panel/RightPanel";
import PlayScene from "../src/scene/PlayScene";
import EditorScene from "../src/scene/EditorScene";

export default function Editor() {
  const setSelected = useStore((state) => state.setSelected);
  const setId = useStore((state) => state.setId);
  const setPlayMode = useStore((state) => state.setPlayMode);
  const playMode = useStore((state) => state.playMode);

  const setScene = useScene((state) => state.setScene);

  useHotkeys();

  useEffect(() => {
    const queryString = window.location.search;
    const urlParams = new URLSearchParams(queryString);
    setId(urlParams.get("scene"));

    setScene({});
    setSelected(null, null);
  }, [setId, setScene, setSelected]);

  return (
    <div className="App">
      {playMode ? (
        <span>
          <Canvas raycaster={RAYCASTER_SETTINGS}>
            <Physics>
              <Debug>
                <PlayScene />
              </Debug>
            </Physics>
          </Canvas>

          <IconButton
            onClick={() => setPlayMode(false)}
            style={{
              position: "absolute",
              top: "20px",
              right: "20px",
            }}
          >
            <CloseIcon fontSize="large" />
          </IconButton>
        </span>
      ) : (
        <span>
          <Navbar />

          <Split
            className="split App"
            gutterSize={6}
            sizes={[80, 20]}
            gutterAlign="end"
          >
            <Canvas
              onPointerMissed={() => setSelected(null, null)}
              gl={{ preserveDrawingBuffer: true }}
            >
              <Physics>
                <EditorScene />
              </Physics>
            </Canvas>

            <RightPanel />
          </Split>
        </span>
      )}
    </div>
  );
}
