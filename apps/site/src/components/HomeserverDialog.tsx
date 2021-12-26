import { useState } from "react";
import {
  Button,
  Dialog,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Grid,
  Radio,
  TextField,
  Typography,
} from "@mui/material";

function validateUrl(url) {
  if (!url || url.length === 0) return false;

  return true;
}

interface Props {
  open: boolean;
  handleClose: () => void;
  setHomeserver: (homeserver: string) => void;
}

export default function HomeserverDialog({
  open,
  handleClose,
  setHomeserver,
}: Props) {
  const [selectedValue, setSelectedValue] = useState("default");
  const [customServer, setCustomServer] = useState("");

  function handleRadioChange(event) {
    setSelectedValue(event.target.value);
  }

  function handleCustomChange(e) {
    setCustomServer(e.target.value);
  }

  function handleHandleClose() {
    if (selectedValue === "default") {
      setHomeserver("matrix.org");
    } else {
      const valid = validateUrl(customServer);
      if (valid) {
        setHomeserver(customServer);
      } else {
        setHomeserver("matrix.org");
        setSelectedValue("default");
      }
    }

    handleClose();
  }

  return (
    <Dialog open={open} onClose={handleHandleClose}>
      <DialogTitle>Sign in to your homeserver</DialogTitle>
      <DialogContent>
        <Grid container direction="column" rowSpacing={5}>
          <Grid item>
            <DialogContentText>
              We call the places where you can host your account {'"'}
              homeservers{'"'}. Matrix.org is the biggest public homeserver in
              the world, so it{"'"}s a good place for many.
            </DialogContentText>
          </Grid>

          <Grid item container direction="column" rowSpacing={2}>
            <Grid item container alignItems="center" columnSpacing={1}>
              <Grid item>
                <Radio
                  checked={selectedValue === "default"}
                  onChange={handleRadioChange}
                  value="default"
                />
              </Grid>
              <Grid item onClick={() => setSelectedValue("default")}>
                <Typography>matrix.org</Typography>
              </Grid>
            </Grid>

            <Grid item container alignItems="center" columnSpacing={1}>
              <Grid item>
                <Radio
                  checked={selectedValue === "custom"}
                  onChange={handleRadioChange}
                  value="custom"
                />
              </Grid>
              <Grid item>
                <TextField
                  onFocus={() => setSelectedValue("custom")}
                  value={customServer}
                  onChange={handleCustomChange}
                  variant="standard"
                  placeholder="Other homeserver"
                />
              </Grid>
            </Grid>
          </Grid>

          <Grid item>
            <Button
              onClick={handleHandleClose}
              variant="contained"
              style={{ width: "100%" }}
            >
              Continue
            </Button>
          </Grid>
        </Grid>
      </DialogContent>
    </Dialog>
  );
}
