import { Button, Stack } from "@mui/material";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import Link from "next/link";

export function SidebarButton({
  emoji = "",
  text = "",
  href = "",
  external = false,
}) {
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
