import { Paper } from "@mui/material";
import { SceneObject } from "3d";

import { useStore } from "../../state/useStore";

import Assets from "./Assets";
import Inspect from "./Inspect";

export default function RightPanel() {
  const selected: SceneObject = useStore((state) => state.selected);

  return (
    <Paper
      square
      variant="outlined"
      style={{ width: "100%", padding: "1rem", border: 0 }}
    >
      {selected ? <Inspect key={selected.id} object={selected} /> : <Assets />}
    </Paper>
  );
}
