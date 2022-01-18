import { ReactChild, useState } from "react";
import { IconButton, Tooltip, useTheme } from "@mui/material";

interface Props {
  tooltip?: string;
  dark?: boolean;
  children?: ReactChild | ReactChild[];
  [x: string]: any;
}

export function ColorIconButton({
  tooltip = "",
  dark = false,
  children,
  ...rest
}: Props) {
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
            ? "rgba(0,0,0,0.2)"
            : undefined,
        }}
        {...rest}
      >
        {children}
      </IconButton>
    </Tooltip>
  );
}
