import { Paper, Typography } from "@mui/material";

export default function RightPanel() {
  return (
    <Paper
      square
      variant="outlined"
      style={{ width: "100%", padding: "1rem", borderTop: 0 }}
    >
      <Typography variant="h3">Scene</Typography>
    </Paper>
  );
}
