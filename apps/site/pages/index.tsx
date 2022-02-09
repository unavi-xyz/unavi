import Link from "next/link";
import Image from "next/image";
import { Grid, Button, Typography, Link as MuiLink } from "@mui/material";

import { useWindowDimensions } from "ui";
import { GITHUB_URL, DISCORD_URL } from "../src/constants";
import Navbar from "../src/components/Navbar";

import awooga from "../public/images/awooga.jpg";

export default function Landing() {
  return (
    <div>
      <Navbar />
      <Body />
    </div>
  );
}

function Body() {
  const { isXs } = useWindowDimensions();

  return (
    <Grid container direction="column">
      <Grid
        item
        style={{
          background:
            "linear-gradient( rgba(0, 0, 0, 0.7), rgba(0, 0, 0, 0.7) ), url(images/street.png)",
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
                An open source, decentralized VR social platform
              </Typography>
            </Grid>
            <Grid item>
              <Link href="/home" passHref>
                <Button
                  variant="outlined"
                  color="secondary"
                  size="large"
                  style={{ margin: "auto", fontSize: "2ch" }}
                >
                  Play Now
                </Button>
              </Link>
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
                The Wired is built on top of{" "}
                <MuiLink
                  href="https://ceramic.network/"
                  target="_blank"
                  color="secondary"
                >
                  Ceramic
                </MuiLink>
                , a decentralized and open source network for sharing data.
              </Typography>
            </Grid>
            <Grid item>
              <Typography variant="h6" color="primary">
                The effect of this decentralization is a platform{" "}
                <strong>controlled by the users</strong>. There is no company
                that decides what is and isn{"'"}t allowed.
              </Typography>
            </Grid>
            <Grid item>
              <Typography variant="h6" color="primary">
                Anyone can host their own server and join the Ceramic network,
                giving you full control over your data.
              </Typography>
            </Grid>
          </Grid>

          <Grid item style={{ marginTop: "32px" }}>
            <Typography variant="h3">🤳 Interoperable</Typography>
          </Grid>
          <Grid item container direction="column" rowSpacing={2}>
            <Grid item>
              <Typography variant="h6" color="primary">
                Your account is stored as a decentralized identifier (
                <MuiLink
                  href="https://thenewstack.io/did-you-hear-decentralized-identifiers-are-coming/"
                  target="_blank"
                  color="secondary"
                >
                  DID
                </MuiLink>
                ).
              </Typography>
            </Grid>
            <Grid item>
              <Typography variant="h6" color="primary">
                This means your identity, and the data you create, is not
                limited to this platform. For example, you could log in to a
                chat app using your DID, and all of your friends from The Wired
                would show up as contacts.
              </Typography>
            </Grid>
          </Grid>

          <Grid item style={{ marginTop: "32px" }}>
            <Typography variant="h3">📂 Open Source</Typography>
          </Grid>
          <Grid item container direction="column" rowSpacing={2}>
            <Grid item>
              <Typography variant="h6" color="primary">
                All code is{" "}
                <MuiLink href={GITHUB_URL} target="_blank" color="secondary">
                  open source
                </MuiLink>
                . The repository is open for contributions, or you can fork it
                and build your own client.
              </Typography>
            </Grid>
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
          <div
            style={{
              border: "solid grey 2px",
              lineHeight: "0px",
              marginTop: isXs ? "32px" : "0px",
            }}
          >
            <Image src={awooga} alt="anime girls" />
          </div>
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
