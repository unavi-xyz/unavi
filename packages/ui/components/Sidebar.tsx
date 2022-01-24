import { ReactChild, useContext, useState, useEffect } from "react";
import { Button, ClickAwayListener, Collapse, Stack } from "@mui/material";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import Link from "next/link";
import useSWR from "swr";
import { ClientContext } from "matrix";

import { useIdenticon } from "../hooks/useIdenticon";
import { getHomeUrl, SidebarButton } from "..";

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
  const { loggedIn, userId, client, logout } = useContext(ClientContext);

  const identicon = useIdenticon(userId);

  async function fetcher(id: string) {
    const profile = await client.getProfileInfo(id);
    const picture = client.mxcUrlToHttp(profile.avatar_url ?? "");
    return { profile, picture };
  }

  const { data } = useSWR(userId, fetcher);

  const [open, setOpen] = useState(false);
  const [homeUrl, setHomeUrl] = useState("");

  useEffect(() => {
    setHomeUrl(getHomeUrl());
  }, []);

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

              <SidebarButton
                text="View Profile"
                href={`${homeUrl}/user/${userId}`}
              />
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
                    src={
                      data?.picture === ""
                        ? identicon
                        : data?.picture ?? identicon
                    }
                    alt="profile picture"
                    style={{
                      height: "3rem",
                      width: "3rem",
                      border: "1px solid black",
                    }}
                  />

                  <Stack>
                    <div>{data?.profile.displayname ?? userId}</div>
                  </Stack>

                  <MoreHorizIcon />
                </Stack>
              </Button>
            </ClickAwayListener>
          </Stack>
        </span>
      ) : (
        <Link href={`${homeUrl}/login`} passHref>
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
