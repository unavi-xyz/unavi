import { useState } from "react";
import { Stack, Typography, useTheme } from "@mui/material";
import EditIcon from "@mui/icons-material/Edit";

export default function SceneName() {
  const theme = useTheme();

  const [hover, setHover] = useState(false);

  function handleMouseOver() {
    setHover(true);
  }

  function handleMouseOut() {
    setHover(false);
  }

  return (
    <Stack
      className="clickable"
      direction="row"
      alignItems="center"
      spacing={1}
      style={{ marginLeft: 5 }}
    >
      <Typography variant="h6">New Scene</Typography>

      <EditIcon
        className="NavbarIcon"
        onMouseOver={handleMouseOver}
        onMouseOut={handleMouseOut}
        style={{
          background: "rgba(0,0,0,0)",
          color: hover ? theme.palette.secondary.main : "rgba(0,0,0,0.2)",
        }}
      />
    </Stack>
  );
}
