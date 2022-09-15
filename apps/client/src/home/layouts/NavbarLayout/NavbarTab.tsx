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
    ? "bg-primaryContainer text-onPrimaryContainer"
    : "hover:bg-surfaceVariant hover:text-onSurfaceVariant";

  return (
    <Link href={href} passHref>
      <div
        className={`cursor-pointer rounded-lg px-4 py-1 font-bold transition ${selectedClass}`}
      >
        {text}
      </div>
    </Link>
  );
}
