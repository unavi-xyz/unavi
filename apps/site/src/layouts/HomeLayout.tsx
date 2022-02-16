import { Box, Grid, Paper, useMediaQuery } from "@mui/material";
import Head from "next/head";

import { theme } from "../theme";
import Sidebar from "../components/sidebar/Sidebar";

export default function HomeLayout({ children }) {
  const md = useMediaQuery(theme.breakpoints.up("md"));

  return (
    <div>
      <Head>
        <title>The Wired - Home</title>
      </Head>

      <Grid container sx={{ maxWidth: "1400px", margin: "auto" }}>
        <Box
          sx={{
            width: md ? "260px" : null,
            paddingLeft: md ? 4 : 0,
            padding: md ? 0 : 1,
          }}
        >
          <Sidebar />
        </Box>

        <Grid item xs>
          <Paper square variant="outlined" style={{ height: "100vh" }}>
            {children}
          </Paper>
        </Grid>

        {md && <Box sx={{ width: "260px" }} />}
      </Grid>
    </div>
  );
}
