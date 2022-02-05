import { Stack, Typography } from "@mui/material";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import MoreHorizIcon from "@mui/icons-material/MoreHoriz";
import { useRouter } from "next/router";

import { ColorIconButton } from "./ColorIconButton";

interface Props {
  text?: string;
  back?: boolean;
  more?: undefined | (() => void);
}

export function BackNavbar({ text = "", back = true, more }: Props) {
  const router = useRouter();

  return (
    <Stack
      direction="row"
      alignItems="center"
      justifyContent="space-between"
      sx={{
        borderBottom: "1px solid rgba(0,0,0,.1)",
        paddingLeft: back ? 0 : 2,
        paddingRight: 1,
        height: "50px",
      }}
    >
      <Stack direction="row" alignItems="center" spacing={1}>
        {back && (
          <ColorIconButton onClick={router.back}>
            <ArrowBackIosNewIcon />
          </ColorIconButton>
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
