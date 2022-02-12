import { useEffect, useState } from "react";
import { Grid, IconButton, Stack, Typography } from "@mui/material";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import Link from "next/link";

interface Props {
  text?: string;
  emoji?: string;
  href?: string;
  back?: boolean;
  more?: undefined | (() => void);
}

export function HomeNavbar({ text = "", emoji = "", href, back, more }: Props) {
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
    <Grid
      container
      direction="row"
      alignItems="center"
      sx={{
        borderBottom: "1px solid rgba(0,0,0,.1)",
        paddingLeft: back || href ? 0.5 : 3,
        paddingRight: 1,
        height: "50px",
      }}
    >
      {(back || href) && (
        <Grid item xs>
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
        </Grid>
      )}

      <Grid
        item
        xs
        container
        justifyContent={back || href ? "center" : "flex-start"}
      >
        <Stack direction="row" spacing={1}>
          <Typography variant="h6" align="center">
            {emoji}
          </Typography>

          <Typography variant="h6" align="center">
            {text}
          </Typography>
        </Stack>
      </Grid>

      <Grid item xs container justifyContent="flex-end">
        {more && (
          <IconButton onClick={more}>
            <MoreHorizIcon />
          </IconButton>
        )}
      </Grid>
    </Grid>
  );
}
