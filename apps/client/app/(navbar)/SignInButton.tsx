"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { useRouter } from "next/navigation";
import { useEffect, useTransition } from "react";

import { useSession } from "../../src/client/auth/useSession";
import RainbowkitWrapper from "./RainbowkitWrapper";

export default function SignInButton() {
  return (
    <RainbowkitWrapper>
      <Button />
    </RainbowkitWrapper>
  );
}

function Button() {
  const { openConnectModal } = useConnectModal();
  const { status } = useSession();
  const router = useRouter();
  const [isLoading, startTransition] = useTransition();

  useEffect(() => {
    if (status !== "authenticated") return;

    startTransition(() => {
      router.refresh();
    });
  }, [status, router, startTransition]);

  return (
    <button
      className={`rounded-full bg-neutral-900 px-6 py-1.5 font-bold text-white outline-neutral-400 transition hover:scale-105 active:scale-100 ${
        isLoading ? "opacity-70" : ""
      }`}
      disabled={isLoading}
      onClick={openConnectModal}
    >
      Sign in
    </button>
  );
}
