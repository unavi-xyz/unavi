import React from "react";
import { Grid, Paper } from "@mui/material";
import Head from "next/head";

import Sidebar from "../components/Sidebar";

export default function HomeLayout({ children }) {
  return (
    <div>
      <Head>
        <title>The Wired - Home</title>
      </Head>

      <Grid container>
        <Grid item xs={4} style={{ maxWidth: "320px" }}>
          <Sidebar />
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
