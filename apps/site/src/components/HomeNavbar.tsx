import { useState } from "react";
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

// import { user } from "../common/gun/gun";
// import { logout } from "../common/gun/auth";
// import useAlias from "../common/gun/hooks/useAlias";
import { useWindowDimensions } from "../hooks";
// import LoginLogoutButton from "../common/components/LoginLogoutButton";

export default function HomeNavbar() {
  // const alias = useAlias(user.is?.pub);

  const { isMobile } = useWindowDimensions();

  const [open, setOpen] = useState(false);

  // function handleLogout() {
  //   logout();
  //   window.location.reload();
  // }

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
                <List>
                  <ListItem>
                    <Link href="worlds">Worlds</Link>
                  </ListItem>
                </List>
                <List>
                  <ListItem>
                    <Link href="friends">Friends</Link>
                  </ListItem>
                </List>
                <List>
                  <ListItem>
                    <Link href="avatars">Avatars</Link>
                  </ListItem>
                </List>
                <Divider />
                <List>
                  <ListItem button>Log Out</ListItem>
                </List>
              </Box>
            </Drawer>
          </div>
        ) : (
          <Grid item xs container justifyContent="flex-start" columnSpacing={1}>
            <Grid item>
              <Link href="/home/worlds" passHref>
                <Button>Worlds</Button>
              </Link>
            </Grid>
            <Grid item>
              <Link href="/home/friends" passHref>
                <Button>Friends</Button>
              </Link>
            </Grid>
            <Grid item>
              <Link href="/home/avatars" passHref>
                <Button>Avatars</Button>
              </Link>
            </Grid>
            <Grid item>
              <Link href="/editor" passHref>
                <Button>Editor</Button>
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
          columnSpacing={1}
        >
          <Grid item>
            {/* {user.is && (
              <Link className="link" to={`user/${user.is.pub}`}>
                <Typography>{alias}</Typography>
              </Link>
            )} */}
          </Grid>
          {!isMobile && (
            <Grid item style={{ paddingRight: "8px" }}>
              {/* <LoginLogoutButton /> */}
              <Button variant="contained">
                <Link href="/home/login">Login</Link>
              </Button>
            </Grid>
          )}
        </Grid>
      </Grid>
    </Paper>
  );
}
