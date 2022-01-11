import { ReactChild, useContext, useState } from "react";
import { Button, ClickAwayListener, Collapse, Stack } from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import Link from "next/link";

import { ClientContext, useMatrixContent, useProfile } from "matrix";
import { useIdenticon } from "../hooks/useIdenticon";
import { SidebarButton } from "..";

interface Props {
  title?: string;
  titleHref?: string;
  loginHref?: string;
  viewProfile?: boolean;
  children: ReactChild | ReactChild[];
}

export function Sidebar({
  title = "The Wired",
  titleHref = "/",
  loginHref = "/login",
  viewProfile = true,
  children,
}: Props) {
  const { loggedIn, userId, client, logout } = useContext(ClientContext);

  const profile = useProfile(client, userId);
  const identicon = useIdenticon(userId);
  const picture = useMatrixContent(profile?.avatar_url);

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
        <div
          style={{
            marginLeft: "auto",
            marginRight: "auto",
          }}
        >
          <Link href={titleHref} passHref>
            <h1>{title}</h1>
          </Link>
        </div>

        {children}
      </Stack>

      {loggedIn ? (
        <span>
          <Collapse in={open}>
            <Stack>
              <Button
                onClick={logout}
                style={{
                  paddingLeft: "30%",
                  justifyContent: "left",
                  fontSize: "1rem",
                }}
              >
                Log Out
              </Button>

              {viewProfile && (
                <SidebarButton
                  text="View Profile"
                  href={`/home/user/${userId}`}
                />
              )}
            </Stack>
          </Collapse>

          <Stack>
            <ClickAwayListener onClickAway={handleClose}>
              <Button onClick={handleToggle} style={{ textTransform: "none" }}>
                <Stack
                  direction="row"
                  alignItems="center"
                  justifyContent="space-between"
                  spacing={1}
                  style={{ width: "100%", fontSize: "1rem", padding: ".2rem" }}
                >
                  <img
                    src={picture ?? identicon}
                    alt="profile picture"
                    style={{
                      height: "3rem",
                      width: "3rem",
                      border: "1px solid black",
                    }}
                  />

                  <Stack>
                    <div>{profile?.displayname ?? userId}</div>
                  </Stack>

                  <MoreHorizIcon />
                </Stack>
              </Button>
            </ClickAwayListener>
          </Stack>
        </span>
      ) : (
        <Link href={loginHref} passHref>
          <Button
            variant="contained"
            style={{
              fontSize: "1rem",
              margin: "2rem",
              borderRadius: 0,
            }}
          >
            Log In
          </Button>
        </Link>
      )}
    </Stack>
  );
}
