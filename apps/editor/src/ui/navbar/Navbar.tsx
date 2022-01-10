import { Button, Grid, Paper, Stack, Typography } from "@mui/material";
import DownloadIcon from "@mui/icons-material/Download";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import EditIcon from "@mui/icons-material/Edit";

import Tools from "./Tools";
import ColorIconButton from "../components/ColorIconButton";

export default function Navbar() {
  return (
    <Paper square variant="outlined" style={{ padding: "0.2rem" }}>
      <Grid container>
        <Grid item xs={4}>
          <Stack
            direction="row"
            alignItems="center"
            justifyContent="flex-start"
            spacing={1}
          >
            <ColorIconButton>
              <ArrowBackIosNewIcon className="NavbarIcon" />
            </ColorIconButton>

            <Stack
              className="clickable"
              direction="row"
              alignItems="center"
              spacing={1}
            >
              <Typography variant="h6">New Scene</Typography>

              <ColorIconButton dark>
                <EditIcon className="NavbarIcon" />
              </ColorIconButton>
            </Stack>
          </Stack>
        </Grid>

        <Grid item xs={4}>
          <Tools />
        </Grid>

        <Grid item xs={4}>
          <Stack direction="row" justifyContent="flex-end" spacing={2}>
            <ColorIconButton tooltip="Download">
              <DownloadIcon className="NavbarIcon" />
            </ColorIconButton>

            <Button
              variant="contained"
              color="secondary"
              size="small"
              style={{
                marginTop: 5,
                marginBottom: 5,
                marginRight: 5,
                paddingTop: 0,
                paddingBottom: 0,
                paddingLeft: 16,
                paddingRight: 16,
              }}
            >
              Publish
            </Button>
          </Stack>
        </Grid>
      </Grid>
    </Paper>
  );
}
