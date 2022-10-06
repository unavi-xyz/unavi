import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";

interface Props {
  href: string;
  text: string;
}

export default function NavigationTab({ href, text }: Props) {
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
      <div
        className={`flex w-full cursor-pointer justify-center rounded-lg py-1 text-lg
                    font-bold transition md:w-min md:px-16 ${selectedClass}`}
      >
        {text}
      </div>
    </Link>
  );
}
