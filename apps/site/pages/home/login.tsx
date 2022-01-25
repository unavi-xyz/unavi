import { useContext, useEffect, useState } from "react";
import {
  Alert,
  Button,
  Divider,
  Grid,
  Snackbar,
  TextField,
  Typography,
} from "@mui/material";
import Link from "next/link";
import Head from "next/head";
import { useRouter } from "next/router";
import { ClientContext } from "matrix";

import HomeserverDialog from "../../src/components/HomeserverDialog";

export default function Login() {
  const router = useRouter();

  const { login, loggedIn } = useContext(ClientContext);

  const [homeserver, setHomeserver] = useState("matrix.org");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [openError, setOpenError] = useState(false);
  const [error, setError] = useState("");

  const [openHomeserver, setOpenHomeserver] = useState(false);

  function handleErrorClose() {
    setOpenError(false);
  }

  async function handleLogin() {
    if (!login) return;

    const err = await login(homeserver, username, password);

    if (!err) {
      router.push("/home");
    } else {
      setError(err.message);
      setOpenError(true);
      return;
    }
  }

  function handleOpenHomeserver() {
    setOpenHomeserver(true);
  }

  function handleCloseHomeserver() {
    setOpenHomeserver(false);
  }

  useEffect(() => {
    if (loggedIn) router.push("/home");
  }, [loggedIn, router]);

  return (
    <div>
      <Head>
        <title>The Wired - Login</title>
      </Head>

      <HomeserverDialog
        open={openHomeserver}
        handleClose={handleCloseHomeserver}
        setHomeserver={setHomeserver}
      />

      <Grid container>
        <Grid
          item
          xs={8}
          container
          direction="column"
          rowSpacing={4}
          style={{ maxWidth: "550px", margin: "auto" }}
        >
          <Grid item container direction="column" rowSpacing={1}>
            <Grid item>
              <Typography variant="h4">Sign in</Typography>
            </Grid>

            <Grid item>
              <Typography variant="h6">Homeserver</Typography>
            </Grid>

            <Grid
              item
              container
              alignItems="center"
              justifyContent="space-between"
            >
              <Grid item>
                <Typography>{homeserver}</Typography>
              </Grid>
              <Grid item>
                <Typography
                  onClick={handleOpenHomeserver}
                  className="link"
                  color="secondary"
                >
                  Edit
                </Typography>
              </Grid>
            </Grid>

            <Grid item>
              <Divider />
            </Grid>
          </Grid>

          <Grid item container direction="column" rowSpacing={2}>
            <Grid item>
              <TextField
                label="Username"
                style={{ width: "100%" }}
                value={username}
                onChange={(e) => setUsername(e.target.value)}
              />
            </Grid>

            <Grid item>
              <TextField
                label="Password"
                type="password"
                style={{ width: "100%" }}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </Grid>
          </Grid>

          <Grid item>
            <Button
              variant="contained"
              onClick={handleLogin}
              style={{ width: "100%" }}
            >
              Sign in
            </Button>
          </Grid>

          <Grid item xs container alignItems="baseline" columnSpacing={0.5}>
            <Grid item>
              <Typography variant="body2">New?</Typography>
            </Grid>
            <Grid item>
              <Typography variant="body2" className="link" color="secondary">
                <a
                  href="https://app.element.io/#/register"
                  target="_blank"
                  rel="noreferrer"
                >
                  Create a matrix account
                </a>
              </Typography>
            </Grid>
          </Grid>

          <Snackbar
            open={openError}
            autoHideDuration={6000}
            onClose={handleErrorClose}
            anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
          >
            <Alert
              onClose={handleErrorClose}
              severity="error"
              sx={{ width: "100%" }}
            >
              {error}
            </Alert>
          </Snackbar>
        </Grid>
      </Grid>
    </div>
  );
}
