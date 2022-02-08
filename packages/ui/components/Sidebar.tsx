import { ReactChild, useContext, useState } from "react";
import {
  Button,
  ClickAwayListener,
  Collapse,
  Stack,
  Typography,
} from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import Link from "next/link";
import { CeramicContext, useProfile } from "ceramic";

import { useIdenticon } from "../hooks/useIdenticon";

interface Props {
  title?: string;
  titleHref?: string;
  children: ReactChild | ReactChild[];
}

export function Sidebar({
  title = "The Wired",
  titleHref = "/",
  children,
}: Props) {
  const { authenticated, id, connect, disconnect } = useContext(CeramicContext);

  const identicon = useIdenticon(id);
  const { profile, imageUrl } = useProfile(id);

  const [open, setOpen] = useState(false);

  function handleToggle() {
    setOpen((prev) => !prev);
  }

  function handleClose() {
    setOpen(false);
  }

  return (
    <Stack justifyContent="space-between" style={{ height: "100%" }}>
      <Stack spacing={1.5}>
        <Link href={titleHref} passHref>
          <h1>{title}</h1>
        </Link>

        {children}
      </Stack>

      {authenticated ? (
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
                        maxWidth: "140px",
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
                        maxWidth: "140px",
                        textOverflow: "ellipsis",
                        overflow: "hidden",
                      }}
                    >
                      {id}
                    </Typography>
                  </Stack>

                  <MoreHorizIcon />
                </Stack>
              </Button>
            </ClickAwayListener>
          </Stack>
        </span>
      ) : (
        <Button
          variant="contained"
          onClick={connect}
          style={{
            fontSize: "1rem",
            margin: "1rem",
            marginBottom: "2rem",
            borderRadius: 0,
          }}
        >
          Connect Wallet
        </Button>
      )}
    </Stack>
  );
}
