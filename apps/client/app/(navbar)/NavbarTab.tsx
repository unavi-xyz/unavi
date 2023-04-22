"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

interface Props {
  href: string;
  text: string;
  exact?: boolean;
  className?: string;
}

export default function NavbarTab({ href, text, exact = false }: Props) {
  const pathname = usePathname();

  const isSelected =
    exact || href === "/"
      ? href === pathname
      : typeof href === "string"
      ? pathname?.startsWith(href)
      : false;

  return (
    <Link href={href} className={`text-lg font-bold ${isSelected ? "" : "text-neutral-400"}`}>
      {text}
    </Link>
  );
}
