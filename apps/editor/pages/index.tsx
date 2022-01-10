import { useState } from "react";
import { Grid, IconButton, Stack, Tooltip, Typography } from "@mui/material";
import AddBoxOutlinedIcon from "@mui/icons-material/AddBoxOutlined";

export default function Friends() {
  const [scenes, setScenes] = useState([]);

  return (
    <Grid className="page" container direction="column" rowSpacing={4}>
      <Grid item>
        <Stack direction="row" alignItems="center" spacing={1}>
          <Typography variant="h2">🌲 Scenes</Typography>

          <span>
            <Tooltip title="New Scene">
              <IconButton>
                <AddBoxOutlinedIcon />
              </IconButton>
            </Tooltip>
          </span>
        </Stack>
      </Grid>

      <Grid item>
        {scenes.length === 0 ? (
          <div>
            <Typography>
              It looks like you don{"'"}t have any scenes.
            </Typography>
            <Typography>
              <Typography className="link" color="secondary" component="span">
                Click Here
              </Typography>{" "}
              to get started.
            </Typography>
          </div>
        ) : null}
      </Grid>
    </Grid>
  );
}
