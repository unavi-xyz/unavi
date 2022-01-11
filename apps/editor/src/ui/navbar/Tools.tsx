import { useState } from "react";
import { IconButton, Stack, Tooltip } from "@mui/material";

import TranslateIcon from "../icons/TranslateIcon";
import RotateIcon from "../icons/RotateIcon";
import ScaleIcon from "../icons/ScaleIcon";

const TOOLS = {
  translate: "Translate",
  rotate: "Rotate",
  scale: "Scale",
};

export default function Tools() {
  const [selected, setSelected] = useState(TOOLS.translate);

  return (
    <Stack direction="row" justifyContent="center" spacing={2}>
      <Tooltip title="Translate">
        <IconButton
          onClick={() => setSelected(TOOLS.translate)}
          style={{
            borderRadius: 5,
            background:
              selected === TOOLS.translate ? "rgba(0, 0, 0, 0.05)" : null,
          }}
        >
          <TranslateIcon className="NavbarIcon" />
        </IconButton>
      </Tooltip>

      <Tooltip title="Rotate">
        <IconButton
          onClick={() => setSelected(TOOLS.rotate)}
          style={{
            borderRadius: 5,
            background:
              selected === TOOLS.rotate ? "rgba(0, 0, 0, 0.05)" : null,
          }}
        >
          <RotateIcon className="NavbarIcon" />
        </IconButton>
      </Tooltip>

      <Tooltip title="Scale">
        <IconButton
          onClick={() => setSelected(TOOLS.scale)}
          style={{
            borderRadius: 5,
            background: selected === TOOLS.scale ? "rgba(0, 0, 0, 0.05)" : null,
          }}
        >
          <ScaleIcon className="NavbarIcon" />
        </IconButton>
      </Tooltip>
    </Stack>
  );
}
