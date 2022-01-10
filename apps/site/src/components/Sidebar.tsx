import { useContext, useEffect, useState } from "react";
import {
  Button,
  ClickAwayListener,
  Collapse,
  Divider,
  Stack,
} from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import Link from "next/link";
import { ClientContext, useMatrixContent, useProfile } from "matrix";

import { useIdenticon } from "../hooks";
import { getAppUrl, getEditorUrl } from "../helpers";

export default function Sidebar() {
  const { loggedIn, userId, client, logout } = useContext(ClientContext);

  const profile = useProfile(client, userId);
  const identicon = useIdenticon(userId);
  const picture = useMatrixContent(profile?.avatar_url);

  const [appUrl, setAppUrl] = useState("/");
  const [editorUrl, setEditorUrl] = useState("/");

  const [open, setOpen] = useState(false);

  function handleToggle() {
    setOpen((prev) => !prev);
  }

  function handleClose() {
    setOpen(false);
  }

  useEffect(() => {
    setAppUrl(getAppUrl());
    setEditorUrl(getEditorUrl());
  }, []);

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

        <Divider />

        <SideButton emoji="🎮" text="Play" href={appUrl} external />
        <SideButton emoji="🚧" text="Editor" href={editorUrl} external />
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

function SideButton({ emoji = "", text = "", href = "", external = false }) {
  if (external) {
    return (
      <Button
        href={href}
        target="_blank"
        style={{
          paddingLeft: "30%",
          paddingRight: "2rem",
          fontSize: "1rem",
          borderRadius: 0,
          justifyContent: "space-between",
        }}
      >
        <Stack direction="row" style={{ justifyContent: "left" }}>
          {emoji && <span style={{ width: "2rem" }}>{emoji}</span>}
          <span>{text}</span>
        </Stack>

        <OpenInNewIcon />
      </Button>
    );
  }

  return (
    <Link href={href} passHref>
      <Button
        style={{
          paddingLeft: "30%",
          fontSize: "1rem",
          justifyContent: "left",
          borderRadius: 0,
        }}
      >
        {emoji && <span style={{ width: "2rem" }}>{emoji}</span>}
        <span>{text}</span>
      </Button>
    </Link>
  );
}
