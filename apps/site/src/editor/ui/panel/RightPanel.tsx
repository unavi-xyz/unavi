import { Paper } from "@mui/material";

import { useStore } from "../../hooks/useStore";

import Packs from "./packs/Packs";
import Inspect from "./inspect/Inspect";

export default function RightPanel() {
  const selected = useStore((state) => state.selected);

  return (
    <Paper
      square
      variant="outlined"
      style={{ width: "100%", padding: "1rem", border: 0 }}
    >
      {selected && <Inspect key={selected.id} />}

      <div style={{ visibility: selected ? "hidden" : "visible" }}>
        <Packs />
      </div>
    </Paper>
  );
}
