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
      <Grid container sx={{ maxWidth: "1400px", margin: "auto" }}>
        <Grid item sx={{ width: "260px" }}>
          <Sidebar title="Editor">
            <SidebarButton emoji="🌲" text="Scenes" href="/" />
            <SidebarButton emoji="🏠" text="Home" href={homeUrl} />
          </Sidebar>
        </Grid>

        <Grid item xs>
          <Paper square variant="outlined" style={{ height: "100vh" }}>
            {children}
          </Paper>
        </Grid>

        <Grid item sx={{ width: "260px" }}></Grid>
      </Grid>
    </div>
  );
}
