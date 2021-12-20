import { useState } from "react";
import Link from "next/link";
import {
  Alert,
  Button,
  Grid,
  Snackbar,
  TextField,
  Typography,
} from "@mui/material";

import { login, signup } from "../../src/matrix/auth";

export default function Login() {
  const [homeserver, setHomeserver] = useState("matrix.org");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [openError, setOpenError] = useState(false);
  const [error, setError] = useState("");

  const disableLogin =
    homeserver.length === 0 || username.length === 0 || password.length === 0;

  function handleErrorClose() {
    setOpenError(false);
  }

  async function handleLogin() {
    const err = await login(homeserver, username, password);
    if (err) {
      setError(err.message);
      setOpenError(true);
      return;
    }
  }

  async function handleSignup() {
    signup(homeserver, username, password);
  }

  return (
    <Grid className="container" container justifyContent="center">
      <Grid
        item
        sm={8}
        md={6}
        container
        direction="column"
        rowSpacing={4}
        justifyContent="center"
        style={{ height: "100vh" }}
      >
        <Grid item>
          <Typography variant="h2">Login</Typography>
        </Grid>
        <Grid item>
          <TextField
            label="Home Server"
            style={{ width: "100%" }}
            value={homeserver}
            onChange={(e) => setHomeserver(e.target.value)}
          />
        </Grid>
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

        <Grid item container columnSpacing={4}>
          <Grid item>
            <Button disabled={disableLogin} onClick={handleLogin}>
              Log In
            </Button>
          </Grid>
          <Grid item xs>
            <Button disabled={disableLogin} onClick={handleSignup}>
              Sign Up
            </Button>
          </Grid>
          <Grid item>
            <Button>
              <Link href="/home">Continue as Guest</Link>
            </Button>
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
  );
}
