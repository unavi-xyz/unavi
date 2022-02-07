import { useEffect, useState } from "react";
import { IconButton, Stack, Typography } from "@mui/material";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import Link from "next/link";

interface Props {
  text?: string;
  href?: string;
  back?: boolean;
  more?: undefined | (() => void);
}

export function BackNavbar({ text = "", href, back, more }: Props) {
  const [prevPath, setPrevPath] = useState("/home");

  useEffect(() => {
    const history: string[] = JSON.parse(
      sessionStorage.getItem("pathHistory") ?? "[]"
    );
    const value = history[history.length - 2] ?? "/home";
    setPrevPath(value);
  });

  function onBack() {
    const history: string[] = JSON.parse(
      sessionStorage.getItem("pathHistory") ?? "[]"
    );
    const newHistory = history.slice(0, -2);
    sessionStorage.setItem("pathHistory", JSON.stringify(newHistory));
  }

  return (
    <Stack
      direction="row"
      alignItems="center"
      justifyContent="space-between"
      sx={{
        borderBottom: "1px solid rgba(0,0,0,.1)",
        paddingLeft: back || href ? 0.5 : 2,
        paddingRight: 1,
        height: "50px",
      }}
    >
      <Stack direction="row" alignItems="center" spacing={1}>
        {back && (
          <Link href={prevPath} passHref>
            <IconButton onClick={onBack}>
              <ArrowBackIosNewIcon />
            </IconButton>
          </Link>
        )}

        {href && (
          <Link href={href} passHref>
            <IconButton>
              <ArrowBackIosNewIcon />
            </IconButton>
          </Link>
        )}

        <Typography variant="h5">{text}</Typography>
      </Stack>

      {more && (
        <IconButton onClick={more}>
          <MoreHorizIcon />
        </IconButton>
      )}
    </Stack>
  );
}
