import { useEffect, useState } from "react";
import { Divider, Grid, Paper } from "@mui/material";
import Head from "next/head";

import { Sidebar, SidebarButton, getAppUrl, getEditorUrl } from "ui";

export default function HomeLayout({ children }) {
  const [appUrl, setAppUrl] = useState("/");
  const [editorUrl, setEditorUrl] = useState("/");

  useEffect(() => {
    setAppUrl(getAppUrl());
    setEditorUrl(getEditorUrl());
  }, []);

  return (
    <div>
      <Head>
        <title>The Wired - Home</title>
      </Head>

      <Grid container>
        <Grid item xs={4} style={{ maxWidth: "320px" }}>
          <Sidebar titleHref="/home" loginHref="/home/login">
            <SidebarButton emoji="🏠" text="Home" href="/home" />
            <SidebarButton emoji="🌏" text="Worlds" href="/home/worlds" />
            <SidebarButton emoji="🚪" text="Rooms" href="/home/rooms" />
            <SidebarButton emoji="🤝" text="Friends" href="/home/friends" />
            <SidebarButton emoji="💃" text="Avatars" href="/home/avatars" />

            <Divider />

            <SidebarButton emoji="🎮" text="Play" href={appUrl} external />
            <SidebarButton emoji="🚧" text="Editor" href={editorUrl} external />
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
