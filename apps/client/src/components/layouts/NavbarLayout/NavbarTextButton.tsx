import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

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

  const bgColor = selected ? "bg-neutral-200" : "";

  return (
    <Link href={href} passHref>
      <button
        className={`hover:bg-neutral-200 rounded-md px-2 py-1 text-sm font-bold
                      transition-all duration-100 ${bgColor}`}
      >
        {text}
      </button>
    </Link>
  );
}
