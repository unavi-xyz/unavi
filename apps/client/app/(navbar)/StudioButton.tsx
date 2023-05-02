"use client";

import Link from "next/link";

import { useAuth } from "@/src/client/AuthProvider";
import { env } from "@/src/env.mjs";

export default function StudioButton() {
  const { status } = useAuth();

  if (!env.NEXT_PUBLIC_HAS_DATABASE || !env.NEXT_PUBLIC_HAS_S3) return null;
  if (status !== "authenticated") return null;

  return (
    <div className="hidden lg:block">
      <Link href="/studio" className="text-lg font-bold text-neutral-400">
        Studio
      </Link>
    </div>
  );
}
