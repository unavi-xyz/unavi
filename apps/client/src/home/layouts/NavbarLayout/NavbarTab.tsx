import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

interface Props {
  href: string;
  text: string;
}

export default function NavbarTab({ href, text }: Props) {
  const router = useRouter();
  const [selected, setSelected] = useState(false);

  useEffect(() => {
    if (href === "/") {
      setSelected(router.asPath === "/");
      return;
    }

    setSelected(router.asPath.startsWith(href));
  }, [router, href]);

  const selectedClass = selected
    ? "bg-neutral-200 hover:bg-neutral-300 active:bg-neutral-400"
    : "hover:bg-neutral-200 active:bg-neutral-300";

  return (
    <Link
      href={href}
      className={`cursor-pointer rounded-lg px-4 py-1 text-lg font-bold transition ${selectedClass}`}
    >
      {text}
    </Link>
  );
}
