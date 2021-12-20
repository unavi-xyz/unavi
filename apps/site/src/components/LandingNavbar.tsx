import { useState } from "react";
import Link from "next/link";
import {
  Grid,
  Button,
  IconButton,
  Drawer,
  ListItem,
  List,
  Divider,
  Box,
  Paper,
} from "@mui/material";
import TwitterIcon from "@mui/icons-material/Twitter";
import GitHubIcon from "@mui/icons-material/GitHub";
import MenuIcon from "@mui/icons-material/Menu";
import DiscordIcon from "./DiscordIcon";
import MediumIcon from "./MediumIcon";

import {
  DISCORD_URL,
  TWITTER_URL,
  MEDIUM_URL,
  GITHUB_URL,
  DOCS_URL,
} from "../constants";
import { useWindowDimensions } from "../hooks";

export default function WelcomeNavbar() {
  const [open, setOpen] = useState(false);

  const { isMobile } = useWindowDimensions();

  const toggleDrawer = (newOpen) => (event) => {
    if (
      event.type === "keydown" &&
      (event.key === "Tab" || event.key === "Shift")
    ) {
      return;
    }

    setOpen(newOpen);
  };

  return (
    <Paper
      square
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        width: "100vw",
        zIndex: 1,
      }}
    >
      <Grid
        container
        className="container"
        style={{
          paddingTop: "0.5vh",
          paddingBottom: "0.5vh",
        }}
      >
        <Grid item xs container justifyContent="flex-start" columnSpacing={1}>
          <Grid item style={{ paddingRight: "8px" }}>
            <IconButton href={TWITTER_URL} target="_blank">
              <TwitterIcon color="primary" />
            </IconButton>
          </Grid>
          <Grid item style={{ paddingRight: "8px" }}>
            <IconButton href={DISCORD_URL} target="_blank">
              <DiscordIcon color="primary" />
            </IconButton>
          </Grid>
          <Grid item style={{ paddingRight: "8px" }}>
            <IconButton href={GITHUB_URL} target="_blank">
              <GitHubIcon color="primary" />
            </IconButton>
          </Grid>
          <Grid item style={{ paddingRight: "8px" }}>
            <IconButton href={MEDIUM_URL} target="_blank">
              <MediumIcon color="primary" />
            </IconButton>
          </Grid>
        </Grid>

        {isMobile ? (
          <div>
            <IconButton onClick={toggleDrawer(true)}>
              <MenuIcon color="primary" />
            </IconButton>
            <Drawer anchor={"left"} open={open} onClose={toggleDrawer(false)}>
              <Box
                sx={{
                  width: 250,
                }}
                role="presentation"
                onClick={toggleDrawer(false)}
                onKeyDown={toggleDrawer(false)}
              >
                <List>
                  <ListItem button>
                    <Link href="/home/login">Log In</Link>
                  </ListItem>
                </List>
                <Divider />
                <List>
                  <ListItem button href={DOCS_URL} target="_blank">
                    Docs
                  </ListItem>
                </List>
              </Box>
            </Drawer>
          </div>
        ) : (
          <Grid item xs container justifyContent="flex-end" columnSpacing={1}>
            <Grid item style={{ paddingRight: "8px" }}>
              <Button href={DOCS_URL} target="_blank">
                Docs
              </Button>
            </Grid>
            <Grid item style={{ paddingRight: "8px" }}>
              <Button variant="contained">
                <Link href="/home/login">Log In</Link>
              </Button>
            </Grid>
          </Grid>
        )}
      </Grid>
    </Paper>
  );
}
