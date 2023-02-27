"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

interface Props {
  href: string;
  text: string;
  exact?: boolean;
  className?: string;
}

export default function NavbarTab({ href, text, exact = false, className }: Props) {
  const pathname = usePathname();
  const isSelected = exact || href === "/" ? href === pathname : pathname?.startsWith(href);

  const selectedClass = isSelected
    ? "bg-neutral-200 hover:bg-neutral-300 active:bg-neutral-400/60"
    : "hover:bg-neutral-200 active:bg-neutral-300";

  return (
    <Link
      href={href}
      className={`cursor-pointer rounded-md px-4 py-1 text-center font-bold transition ${selectedClass} ${className}`}
    >
      {text}
    </Link>
  );
}
