import { useContext, useEffect, useState } from "react";
import { Button } from "@mui/material";
import Link from "next/link";
import { useRouter } from "next/router";
import { CeramicContext } from "ceramic";

export function SidebarButton({
  emoji = "",
  text = "",
  href = "",
  disabled = false,
}) {
  const router = useRouter();

  const { userId: id } = useContext(CeramicContext);

  const [selected, setSelected] = useState(false);

  useEffect(() => {
    if (router.pathname.includes("/home/user") && href.includes("/home/user")) {
      setSelected(router.query.id === id);
    } else {
      setSelected(router.pathname === href);
    }
  }, [router.pathname, router.query.id, href, id]);

  return (
    <Link href={href} passHref>
      <Button
        disabled={disabled}
        style={{
          fontSize: "1rem",
          justifyContent: "left",
          borderRadius: "0px",
        }}
      >
        {emoji && <span style={{ width: "2rem" }}>{emoji}</span>}
        <span
          style={{
            color: "black",
            fontWeight: selected ? "bold" : "inherit",
          }}
        >
          {text}
        </span>
      </Button>
    </Link>
  );
}
