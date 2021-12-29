import { useContext, useState } from "react";
import {
  Alert,
  Button,
  Grid,
  IconButton,
  InputAdornment,
  Snackbar,
  TextField,
  Tooltip,
  Typography,
} from "@mui/material";
import HelpOutlineIcon from "@mui/icons-material/HelpOutline";
import Link from "next/link";
import { useRouter } from "next/router";

import { MatrixContext } from "matrix";

export default function Register() {
  const router = useRouter();

  const { register } = useContext(MatrixContext);

  const [homeserver, setHomeserver] = useState("matrix.org");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [openError, setOpenError] = useState(false);
  const [error, setError] = useState("");

  function handleErrorClose() {
    setOpenError(false);
  }

  async function handleRegister() {
    const err = await register(homeserver, username, password);

    if (!err) {
      router.push("/home");
    } else {
      setError(err.message);
      setOpenError(true);
      return;
    }
  }

  return (
    <Grid container>
      <Grid
        className="container underNavbar"
        item
        sm={8}
        xl={5}
        container
        direction="column"
        rowSpacing={4}
      >
        <Grid item>
          <Typography variant="h2">Create Account</Typography>
        </Grid>

        <Grid item container direction="column" rowSpacing={2}>
          <Grid item>
            <TextField
              label="Home Server"
              style={{ width: "100%" }}
              value={homeserver}
              onChange={(e) => setHomeserver(e.target.value)}
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    <Tooltip title="Learn More" placement="right">
                      <IconButton
                        href="https://the-wired.gitbook.io/the-wired/guides/logging-in"
                        target="_blank"
                      >
                        <HelpOutlineIcon />
                      </IconButton>
                    </Tooltip>
                  </InputAdornment>
                ),
              }}
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
        </Grid>

        <Grid item>
          <Button
            variant="contained"
            onClick={handleRegister}
            style={{ width: "100%" }}
          >
            Create Account
          </Button>
        </Grid>

        <Grid item xs container alignItems="baseline" columnSpacing={0.5}>
          <Grid item>
            <Typography variant="body2">Already have an account?</Typography>
          </Grid>
          <Grid item>
            <Typography variant="body2" className="link" color="secondary">
              <Link href="/home/login">Log in</Link>
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
  );
}
