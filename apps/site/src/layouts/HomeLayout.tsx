import { useContext, useEffect, useState } from "react";
import { Grid, Paper } from "@mui/material";
import Head from "next/head";
import { Sidebar, SidebarButton, getEditorUrl } from "ui";
import { CeramicContext } from "ceramic";

export default function HomeLayout({ children }) {
  const { id } = useContext(CeramicContext);

  const [editorUrl, setEditorUrl] = useState("/");

  useEffect(() => {
    setEditorUrl(getEditorUrl());
  }, []);

  return (
    <div>
      <Head>
        <title>The Wired - Home</title>
      </Head>

      <Grid container sx={{ maxWidth: "1400px", margin: "auto" }}>
        <Grid item sx={{ width: "260px" }}>
          <Sidebar titleHref="/home">
            <SidebarButton emoji="🏠" text="Home" href="/home" />
            <SidebarButton emoji="🌏" text="Worlds" href="/home/worlds" />
            <SidebarButton emoji="🚪" text="Rooms" href="/home/rooms" />
            <SidebarButton emoji="🤝" text="Friends" href="/home/friends" />
            <SidebarButton emoji="💃" text="Avatars" href="/home/avatars" />
            <SidebarButton
              emoji="💎"
              text="Profile"
              href={`/home/user/${id}`}
            />
            <SidebarButton emoji="🚧" text="Editor" href={editorUrl} />
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
