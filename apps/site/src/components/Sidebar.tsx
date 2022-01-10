import { useContext, useRef, useState } from "react";
import { Button, ClickAwayListener, Collapse, Stack } from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import Link from "next/link";

import { ClientContext, useMatrixContent, useProfile } from "matrix";
import { useIdenticon, useWindowDimensions } from "../hooks";

export default function Sidebar() {
  const containerRef = useRef();

  const { isMobile } = useWindowDimensions();

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
          <Link href="/home" passHref>
            <h1>The Wired</h1>
          </Link>
        </div>

        <SideButton emoji="🏠" text="Home" href="/home" />
        <SideButton emoji="🌏" text="Worlds" href="/home/worlds" />
        <SideButton emoji="🚪" text="Rooms" href="/home/rooms" />
        <SideButton emoji="🤝" text="Friends" href="/home/friends" />
        <SideButton emoji="💃" text="Avatars" href="/home/avatars" />
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

              <SideButton text="View Profile" href={`/home/user/${userId}`} />
            </Stack>
          </Collapse>

          <Stack>
            <div ref={containerRef}></div>
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
        <Link href="/home/login" passHref>
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

function SideButton({ emoji = "", text = "", href = "" }) {
  return (
    <Link href={href} passHref>
      <Button
        style={{
          paddingLeft: "30%",
          justifyContent: "left",
          fontSize: "1rem",
        }}
      >
        {emoji && <span style={{ width: "2rem" }}>{emoji}</span>}
        {text && <span>{text}</span>}
      </Button>
    </Link>
  );
}
