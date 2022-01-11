import { Grid, Paper } from "@mui/material";

import { Sidebar, SidebarButton } from "ui";

export default function SidebarLayout({ children }) {
  return (
    <div>
      <Grid container>
        <Grid item xs={4} style={{ maxWidth: "320px" }}>
          <Sidebar title="Editor" viewProfile={false}>
            <SidebarButton emoji="🌲" text="Scenes" href="/" />
          </Sidebar>
        </Grid>

        <Grid item xs>
          <Paper square variant="outlined" style={{ height: "100vh" }}>
            {children}
          </Paper>
        </Grid>
      </Grid>
    </div>
  );
}
