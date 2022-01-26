import { useEffect, useState } from "react";
import { Divider, Grid, Paper } from "@mui/material";

import { getHomeUrl, Sidebar, SidebarButton } from "ui";

export default function SidebarLayout({ children }) {
  const [homeUrl, setHomeUrl] = useState("/");

  useEffect(() => {
    setHomeUrl(getHomeUrl());
  }, []);

  return (
    <div>
      <Grid container>
        <Grid item xs={4} style={{ maxWidth: "320px" }}>
          <Sidebar title="Editor">
            <SidebarButton emoji="🌲" text="Scenes" href="/" />

            <Divider />

            <SidebarButton emoji="🏠" text="Home" href={homeUrl} />
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
