import { IconButton, Stack, Tooltip } from "@mui/material";

import { TOOLS, useStore } from "../../state";
import TranslateIcon from "../icons/TranslateIcon";
import RotateIcon from "../icons/RotateIcon";
import ScaleIcon from "../icons/ScaleIcon";

export default function Tools() {
  const tool = useStore((state) => state.tool);
  const setTool = useStore((state) => state.setTool);

  return (
    <Stack direction="row" justifyContent="center" spacing={2}>
      <Tooltip title="Translate">
        <IconButton
          onClick={() => setTool(TOOLS.translate)}
          style={{
            borderRadius: 5,
            background: tool === TOOLS.translate ? "rgba(0, 0, 0, 0.05)" : null,
          }}
        >
          <TranslateIcon className="NavbarIcon" />
        </IconButton>
      </Tooltip>

      <Tooltip title="Rotate">
        <IconButton
          onClick={() => setTool(TOOLS.rotate)}
          style={{
            borderRadius: 5,
            background: tool === TOOLS.rotate ? "rgba(0, 0, 0, 0.05)" : null,
          }}
        >
          <RotateIcon className="NavbarIcon" />
        </IconButton>
      </Tooltip>

      <Tooltip title="Scale">
        <IconButton
          onClick={() => setTool(TOOLS.scale)}
          style={{
            borderRadius: 5,
            background: tool === TOOLS.scale ? "rgba(0, 0, 0, 0.05)" : null,
          }}
        >
          <ScaleIcon className="NavbarIcon" />
        </IconButton>
      </Tooltip>
    </Stack>
  );
}
