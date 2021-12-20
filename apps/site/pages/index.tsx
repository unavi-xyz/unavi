import Link from "next/link";
import Image from "next/image";
import { Grid, Button, Typography, Link as MuiLink } from "@mui/material";

import WelcomeNavbar from "../src/components/WelcomeNavbar";
import { GITHUB_URL, DISCORD_URL } from "../src/constants";
import { useWindowDimensions } from "../src/hooks";

import awooga from "../public/awooga.jpg";

export default function Welcome() {
  return (
    <div>
      <WelcomeNavbar />
      <Body />
    </div>
  );
}

function Body() {
  const { isMobile } = useWindowDimensions();

  return (
    <Grid container direction="column">
      <Grid
        item
        style={{
          background:
            "linear-gradient( rgba(0, 0, 0, 0.7), rgba(0, 0, 0, 0.7) ), url(street.png)",
          backgroundPosition: "50%",
          height: "min(1220px, 100vh)",
          marginBottom: "10ch",
          marginTop: "5ch",
        }}
      >
        <Grid
          container
          className="container"
          style={{
            paddingTop: "30vh",
            height: "100%",
          }}
        >
          <Grid item container direction="column" rowSpacing={2}>
            <Grid item>
              <Typography
                variant="h2"
                style={{
                  color: "white",
                  fontWeight: "500",
                }}
              >
                The Wired 🔌⚡
              </Typography>
            </Grid>
            <Grid item>
              <Typography variant="h4" style={{ color: "white" }}>
                A decentralized VR social platform owned by its users
              </Typography>
            </Grid>
            <Grid item>
              <Button
                variant="outlined"
                color="secondary"
                style={{ margin: "auto", fontSize: "2ch" }}
              >
                <Link href="/home/login">Join Now</Link>
              </Button>
            </Grid>
          </Grid>

          <Grid item container alignItems="flex-end">
            <svg className="arrows">
              <path className="a1" d="M0 0 L30 32 L60 0"></path>
              <path className="a2" d="M0 20 L30 52 L60 20"></path>
              <path className="a3" d="M0 40 L30 72 L60 40"></path>
            </svg>
          </Grid>
        </Grid>
      </Grid>

      <Grid item container className="container">
        <Grid item xs={12} md={6} container direction="column" rowSpacing={2}>
          <Grid item>
            <Typography variant="h3">💻 Decentralized</Typography>
          </Grid>
          <Grid item container direction="column" rowSpacing={2}>
            <Grid item>
              <Typography variant="h6" color="primary">
                The app is entirely client side, in the browser.
              </Typography>
            </Grid>
            <Grid item>
              <Typography variant="h6" color="primary">
                All data is stored in a decentralized database using{" "}
                <MuiLink
                  href="https://github.com/amark/gun#readme"
                  target="_blank"
                  color="secondary"
                >
                  GunDB
                </MuiLink>
                . Everyone playing the game becomes a node in the network,
                collectively forming a database without reliance on a server.
              </Typography>
            </Grid>
            <Grid item>
              <Typography variant="h6" color="primary">
                This means there is no single source of truth - no company that
                decides what is and isn{"'"}t allowed.
              </Typography>
            </Grid>
          </Grid>

          <Grid item style={{ marginTop: "32px" }}>
            <Typography variant="h3">📂 Open Source</Typography>
          </Grid>
          <Grid item>
            <Typography variant="h6" color="primary">
              All code is{" "}
              <MuiLink href={GITHUB_URL} target="_blank" color="secondary">
                open source
              </MuiLink>
              . The repository is open for contributions, or you can fork it and
              build your own client.
            </Typography>
          </Grid>

          <Grid item style={{ marginTop: "32px" }}>
            <Typography variant="h3">🎉 Join the Community!</Typography>
          </Grid>
          <Grid item>
            <Typography variant="h6" color="primary">
              Come join us on{" "}
              <MuiLink href={DISCORD_URL} target="_blank" color="secondary">
                Discord
              </MuiLink>
              ! It{"'"}s a great place to get involved, whether you want to help
              out on development, or just meet people and have a good time.
            </Typography>
          </Grid>
        </Grid>

        <Grid item xs={12} md={6} style={{ paddingLeft: "24px" }}>
          {!isMobile && (
            <div
              style={{
                border: "solid grey 2px",
                width: "100%",
              }}
            >
              <Image src={awooga} alt="" />
            </div>
          )}
        </Grid>
      </Grid>

      <Grid
        item
        container
        justifyContent="center"
        style={{ marginTop: "200px" }}
      >
        <Typography color="primary">:3</Typography>
      </Grid>
    </Grid>
  );
}
