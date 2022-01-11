import { useEffect, useState } from "react";
import { Grid, Paper } from "@mui/material";

import { Sidebar, SidebarButton, getAppUrl, getEditorUrl } from "ui";

export default function SidebarLayout({ children }) {
  const [appUrl, setAppUrl] = useState("/");
  const [editorUrl, setEditorUrl] = useState("/");

  useEffect(() => {
    setAppUrl(getAppUrl());
    setEditorUrl(getEditorUrl());
  }, []);

  return (
    <div>
      <Grid container>
        <Grid item xs={4} style={{ maxWidth: "320px" }}>
          <Sidebar viewProfile={false}>
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
