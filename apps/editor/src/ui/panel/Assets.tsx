import { Grid, Typography } from "@mui/material";
import { OBJECTS, OBJ_NAMES } from "3d";

import ObjectCard from "./ObjectCard";

export default function Assets() {
  return (
    <div>
      <Typography variant="h3" sx={{ marginBottom: 2 }}>
        Objects
      </Typography>

      <Grid container spacing={2}>
        {Array.from(Object.keys(OBJECTS)).map((name: OBJ_NAMES) => {
          return <ObjectCard key={name} name={name} />;
        })}
      </Grid>
    </div>
  );
}
