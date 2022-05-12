import { useEffect, useState } from "react";
import { useRouter } from "next/router";
import Link from "next/link";

interface Props {
  href: string;
  text: string;
}

export default function NavbarTextButton({ href, text }: Props) {
  const router = useRouter();

  const [selected, setSelected] = useState(false);

  useEffect(() => {
    if (href === "/") {
      setSelected(router.pathname === "/");
      return;
    }

    setSelected(router.pathname.startsWith(href));
  }, [router, href]);

  const selectedClass = selected
    ? "bg-secondaryContainer text-onSecondaryContainer"
    : "hover:bg-surfaceVariant";

  return (
    <Link href={href} passHref>
      <button
        className={`px-3 py-1 rounded-lg text-onBackground font-bold
                    transition  ${selectedClass}`}
      >
        {text}
      </button>
    </Link>
  );
}
