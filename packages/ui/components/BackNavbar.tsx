import { Stack, Typography } from "@mui/material";
import ArrowBackIosNewIcon from "@mui/icons-material/ArrowBackIosNew";
import { useRouter } from "next/router";
import { ColorIconButton } from "./ColorIconButton";

export function BackNavbar({ text = "", back = true }) {
  const router = useRouter();

  return (
    <Stack
      direction="row"
      alignItems="center"
      spacing={1}
      style={{
        borderBottom: "1px solid rgba(0,0,0,.1)",
        paddingLeft: back ? 0 : 12,
        height: "50px",
      }}
    >
      {back && (
        <ColorIconButton onClick={router.back}>
          <ArrowBackIosNewIcon />
        </ColorIconButton>
      )}

      <Typography variant="h5">{text}</Typography>
    </Stack>
  );
}
