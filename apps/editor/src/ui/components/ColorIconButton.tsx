import { useState } from "react";
import { IconButton, Tooltip, useTheme } from "@mui/material";

export default function ColorIconButton({
  tooltip = "",
  dark = false,
  children,
  ...rest
}) {
  const theme = useTheme();

  const [hover, setHover] = useState(false);

  function handleMouseOver() {
    setHover(true);
  }

  function handleMouseOut() {
    setHover(false);
  }

  return (
    <Tooltip title={tooltip}>
      <IconButton
        onMouseOver={handleMouseOver}
        onMouseOut={handleMouseOut}
        style={{
          background: "rgba(0,0,0,0)",
          color: hover
            ? theme.palette.secondary.main
            : dark
            ? "rgba(255,255,255,0.2)"
            : null,
        }}
        {...rest}
      >
        {children}
      </IconButton>
    </Tooltip>
  );
}
