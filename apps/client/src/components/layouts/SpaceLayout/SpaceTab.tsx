import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

interface Props {
  href: string;
  text: string;
}

export default function SpaceTab({ href, text }: Props) {
  const router = useRouter();
  const [selected, setSelected] = useState(false);

  useEffect(() => {
    if (href === "/") {
      setSelected(router.asPath === "/");
      return;
    }

    setSelected(router.asPath == href);
  }, [router, href]);

  const selectedClass = selected
    ? "bg-surfaceVariant"
    : "hover:bg-surfaceVariant";

  return (
    <Link href={href} passHref>
      <button
        className={`w-full md:w-min md:px-16 py-1 rounded-lg text-lg font-bold transition ${selectedClass}`}
      >
        {text}
      </button>
    </Link>
  );
}
