import { useContext, useState } from "react";
import Link from "next/link";
import {
  Button,
  Divider,
  Drawer,
  Grid,
  IconButton,
  List,
  ListItem,
  Paper,
} from "@mui/material";
import { Box } from "@mui/system";
import MenuIcon from "@mui/icons-material/Menu";

import { MatrixContext } from "../matrix/MatrixProvider";
import { useWindowDimensions } from "../hooks";
import LoginButton from "./LoginButton";

export default function HomeNavbar() {
  const { isMobile } = useWindowDimensions();

  const { loggedIn } = useContext(MatrixContext);

  const [open, setOpen] = useState(false);

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
                <Link href="/home" passHref>
                  <List>
                    <ListItem button>🏠 Home</ListItem>
                  </List>
                </Link>
                <Link href="/home/worlds" passHref>
                  <List>
                    <ListItem button>🌏 Worlds</ListItem>
                  </List>
                </Link>
                <Link href="/home/friends" passHref>
                  <List>
                    <ListItem button>🤝 Friends</ListItem>
                  </List>
                </Link>
                <Link href="/home/avatars" passHref>
                  <List>
                    <ListItem button>💃 Avatars</ListItem>
                  </List>
                </Link>
                <Divider />
                <Link href="/home/editor" passHref>
                  <List>
                    <ListItem button>🚧 Editor</ListItem>
                  </List>
                </Link>
              </Box>
            </Drawer>
          </div>
        ) : (
          <Grid
            item
            xs={10}
            container
            justifyContent="flex-start"
            alignItems="center"
            columnSpacing={1}
          >
            <Grid item>
              <Link href="/home" passHref>
                <Button>🏠 Home</Button>
              </Link>
            </Grid>
            <Grid item>
              <Link href="/home/worlds" passHref>
                <Button>🌏 Worlds</Button>
              </Link>
            </Grid>
            <Grid item>
              <Link href="/home/friends" passHref>
                <Button>🤝 Friends</Button>
              </Link>
            </Grid>
            <Grid item>
              <Link href="/home/avatars" passHref>
                <Button>💃 Avatars</Button>
              </Link>
            </Grid>
            <Grid item>
              <Link href="/home/editor" passHref>
                <Button>🚧 Editor</Button>
              </Link>
            </Grid>
          </Grid>
        )}

        <Grid
          item
          xs
          container
          alignItems="center"
          justifyContent="flex-end"
          columnSpacing={2}
        >
          <Grid item>
            {loggedIn && (
              <Link href={`/home/user/id`} passHref>
                <a className="link">user</a>
              </Link>
            )}
          </Grid>
          <Grid item>
            <LoginButton />
          </Grid>
        </Grid>
      </Grid>
    </Paper>
  );
}
