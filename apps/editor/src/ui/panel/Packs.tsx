import { useState } from "react";
import { Grid, IconButton, Stack, Typography } from "@mui/material";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import { PACKS } from "3d";

import ObjectCard from "./ObjectCard";
import PackCard from "./PackCard";

export default function Packs() {
  const [pack, setPack] = useState<null | string>(null);

  if (pack) {
    return (
      <Grid container spacing={2}>
        <Grid item container alignItems="center">
          <Grid item xs={2}>
            <IconButton onClick={() => setPack(null)}>
              <ArrowBackIosNewIcon />
            </IconButton>
          </Grid>

          <Grid item xs={8} container justifyContent="center">
            <Typography variant="h4">{pack}</Typography>
          </Grid>

          <Grid item xs={2}></Grid>
        </Grid>

        <Grid item container spacing={2}>
          {PACKS[pack].map((name) => {
            return <ObjectCard key={name} name={name} />;
          })}
        </Grid>
      </Grid>
    );
  }

  return (
    <Stack spacing={2}>
      <Typography variant="h4">Asset Packs</Typography>

      <Stack spacing={1}>
        {Array.from(Object.keys(PACKS)).map((name) => {
          return <PackCard key={name} name={name} setPack={setPack} />;
        })}
      </Stack>
    </Stack>
  );
}
