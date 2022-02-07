import { useEffect, useState } from "react";
import { Stack, Typography } from "@mui/material";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import Link from "next/link";

import { ColorIconButton } from "./ColorIconButton";

interface Props {
  text?: string;
  href?: string;
  back?: boolean;
  more?: undefined | (() => void);
}

export function BackNavbar({ text = "", href, back, more }: Props) {
  const [prevPath, setPrevPath] = useState("/home");

  useEffect(() => {
    const value = sessionStorage.getItem("prevPath") ?? "/home";
    setPrevPath(value);
  }, []);

  return (
    <Stack
      direction="row"
      alignItems="center"
      justifyContent="space-between"
      sx={{
        borderBottom: "1px solid rgba(0,0,0,.1)",
        paddingLeft: back || href ? 0 : 2,
        paddingRight: 1,
        height: "50px",
      }}
    >
      <Stack direction="row" alignItems="center" spacing={1}>
        {back && (
          <Link href={prevPath} passHref>
            <ColorIconButton>
              <ArrowBackIosNewIcon />
            </ColorIconButton>
          </Link>
        )}

        {href && (
          <Link href={href} passHref>
            <ColorIconButton>
              <ArrowBackIosNewIcon />
            </ColorIconButton>
          </Link>
        )}

        <Typography variant="h5">{text}</Typography>
      </Stack>

      {more && (
        <ColorIconButton onClick={more}>
          <MoreHorizIcon />
        </ColorIconButton>
      )}
    </Stack>
  );
}
