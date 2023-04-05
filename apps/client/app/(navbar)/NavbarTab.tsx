"use client";

import { Route } from "next";
import Link from "next/link";
import { usePathname } from "next/navigation";

interface Props<T extends string> {
  href: Route<T>;
  text: string;
  exact?: boolean;
  className?: string;
}

export default function NavbarTab<T extends string>({
  href,
  text,
  exact = false,
  className,
}: Props<T>) {
  const pathname = usePathname();
  const isSelected =
    exact || href === "/"
      ? href === pathname
      : typeof href === "string"
      ? pathname?.startsWith(href)
      : false;

  const selectedClass = isSelected
    ? "bg-neutral-200 hover:bg-neutral-300 active:bg-neutral-400/60"
    : "hover:bg-neutral-200 active:bg-neutral-300";

  return (
    <Link
      href={href}
      className={`cursor-pointer rounded-xl px-4 py-1 text-center text-lg font-bold transition ${selectedClass} ${className}`}
    >
      {text}
    </Link>
  );
}
