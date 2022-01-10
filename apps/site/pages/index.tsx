import Link from "next/link";
import Image from "next/image";
import { Grid, Button, Typography, Link as MuiLink } from "@mui/material";

import Navbar from "../src/components/Navbar";
import { GITHUB_URL, DISCORD_URL } from "../src/constants";
import { useWindowDimensions } from "../src/hooks";

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
              <Link href="/home/login" passHref>
                <Button
                  variant="outlined"
                  color="secondary"
                  style={{ margin: "auto", fontSize: "2ch" }}
                >
                  Join Now
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
                  href="https://matrix.org/docs/guides/introduction"
                  target="_blank"
                  color="secondary"
                >
                  Matrix
                </MuiLink>
                , an open standard for interoperable and decentralized
                communication.
              </Typography>
            </Grid>
            <Grid item>
              <Typography variant="h6" color="primary">
                Matrix uses a federated network architecture (similar to how
                email works). Each user chooses a homeserver to log in to. These
                servers then communicate with each other, forming a
                decentralized network with no single point of control.
              </Typography>
            </Grid>
            <Grid item>
              <Typography variant="h6" color="primary">
                Importantly, anyone can host their own server and join the
                network, giving you full control over your data.
              </Typography>
            </Grid>
          </Grid>

          <Grid item style={{ marginTop: "32px" }}>
            <Typography variant="h3">🤳 Interoperable</Typography>
          </Grid>
          <Grid item container direction="column" rowSpacing={2}>
            <Grid item>
              <Typography variant="h6" color="primary">
                With the interoperability of Matrix, you have a persistant
                identity across different apps.
              </Typography>
            </Grid>
            <Grid item>
              <Typography variant="h6" color="primary">
                For example, you can friend someone in The Wired, then open a
                chat app later (such as{" "}
                <MuiLink
                  href="https://element.io/personal"
                  target="_blank"
                  color="secondary"
                >
                  Element
                </MuiLink>
                ) and message them. They both use the same Matrix account, with
                the same friends list.
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
