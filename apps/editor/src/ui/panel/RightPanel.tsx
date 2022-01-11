import { Grid, Paper, Typography } from "@mui/material";
import { OBJECTS, OBJ_NAMES } from "3d";

import ObjectCard from "./ObjectCard";

export default function RightPanel() {
  return (
    <Paper
      square
      variant="outlined"
      style={{ width: "100%", padding: "1rem", borderTop: 0 }}
    >
      <Typography variant="h3" sx={{ marginBottom: 2 }}>
        Objects
      </Typography>

      <Grid container spacing={2}>
        {Array.from(Object.keys(OBJECTS)).map((name: OBJ_NAMES) => {
          return <ObjectCard key={name} name={name} />;
        })}
      </Grid>
    </Paper>
  );
}
