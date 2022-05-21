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
    : "hover:bg-surfaceVariant";

  return (
    <Link href={href} passHref>
      <button
        className={`px-3 py-1 rounded-lg font-bold transition ${selectedClass}`}
      >
        {text}
      </button>
    </Link>
  );
}
