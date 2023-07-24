"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { FcGoogle } from "react-icons/fc";

import { env } from "@/src/env.mjs";
import Button from "@/src/ui/Button";
import DialogContent, { DialogRoot, DialogTrigger } from "@/src/ui/Dialog";

import { useSignInStore } from "./signInStore";

interface Props {
  loading?: boolean;
}

export default function SignInButton({ loading }: Props) {
  const open = useSignInStore((state) => state.open);

  return (
    <DialogRoot
      open={open}
      onOpenChange={(value) => useSignInStore.setState({ open: value })}
    >
      <DialogContent title="Sign in to UNAVI">
        <SignInPage />
      </DialogContent>

      <DialogTrigger asChild>
        <button
          className={`rounded-full bg-neutral-900 px-6 py-1.5 font-bold text-white outline-2 outline-offset-4 transition active:scale-100 ${
            loading ? "opacity-70" : "hover:scale-105"
          }`}
          disabled={loading}
        >
          Sign in
        </button>
      </DialogTrigger>
    </DialogRoot>
  );
}

export function SignInPage({ beforeOpen }: { beforeOpen?: () => void }) {
  const { openConnectModal } = useConnectModal();

  function handleWalletLogin() {
    if (openConnectModal) {
      useSignInStore.setState({ open: false });
      if (beforeOpen) beforeOpen();
      openConnectModal();
    }
  }

  return (
    <div className="flex items-stretch justify-between space-x-2">
      <Button onClick={handleWalletLogin} className="h-11 w-full rounded-lg">
        Connect Wallet
      </Button>

      {env.NEXT_PUBLIC_HAS_GOOGLE_OAUTH ? (
        <a
          href="/api/auth/methods/google"
          about="Sign in with Google"
          className="flex h-11 w-full items-center justify-center rounded-lg border border-neutral-400 px-2 transition hover:bg-neutral-100"
        >
          <FcGoogle className="text-2xl" />
        </a>
      ) : null}
    </div>
  );
}
