import { useContext, useState } from "react";
import {
  Button,
  ClickAwayListener,
  Collapse,
  Stack,
  Typography,
} from "@mui/material";
import { LoadingButton } from "@mui/lab";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import { CeramicContext, useProfile } from "ceramic";

import { useIdenticon } from "../../hooks/useIdenticon";

export default function UserProfileButton() {
  const { authenticated, userId, connect, disconnect } =
    useContext(CeramicContext);

  const identicon = useIdenticon(userId);
  const { profile, imageUrl } = useProfile(userId);

  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  function handleToggle() {
    setOpen((prev) => !prev);
  }

  function handleClose() {
    setOpen(false);
  }

  async function handleConnect() {
    setLoading(true);
    await connect();
    setLoading(false);
  }

  if (!authenticated) {
    return (
      <LoadingButton
        loading={loading}
        variant="contained"
        onClick={handleConnect}
        style={{
          fontSize: "1rem",
          margin: "1rem",
          marginBottom: "2rem",
          borderRadius: 0,
        }}
      >
        Connect Wallet
      </LoadingButton>
    );
  }

  return (
    <span>
      <Collapse in={open}>
        <Stack>
          <Button
            onClick={() => {
              handleClose();
              disconnect();
            }}
            style={{
              paddingLeft: "30%",
              justifyContent: "left",
              fontSize: "1rem",
              borderRadius: "0px",
            }}
          >
            Log Out
          </Button>
        </Stack>
      </Collapse>

      <Stack>
        <ClickAwayListener onClickAway={handleClose}>
          <Button
            onClick={handleToggle}
            style={{ textTransform: "none", borderRadius: "0px" }}
          >
            <Stack
              direction="row"
              alignItems="center"
              justifyContent="space-between"
              spacing={1}
              style={{ width: "100%", fontSize: "1rem", padding: ".2rem" }}
            >
              <img
                src={imageUrl ?? identicon}
                alt="profile picture"
                style={{
                  height: "3rem",
                  width: "3rem",
                  border: "1px solid black",
                  objectFit: "cover",
                }}
              />

              <Stack alignItems="start">
                <Typography
                  sx={{
                    color: "black",
                    maxWidth: "110px",
                    textOverflow: "ellipsis",
                    overflow: "hidden",
                  }}
                >
                  {profile?.name}
                </Typography>
                <Typography
                  variant="caption"
                  sx={{
                    color: "GrayText",
                    maxWidth: "110px",
                    textOverflow: "ellipsis",
                    overflow: "hidden",
                  }}
                >
                  {userId}
                </Typography>
              </Stack>

              <MoreHorizIcon />
            </Stack>
          </Button>
        </ClickAwayListener>
      </Stack>
    </span>
  );
}
