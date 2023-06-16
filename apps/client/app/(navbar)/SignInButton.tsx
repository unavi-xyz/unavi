"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { useState } from "react";
import { FcGoogle } from "react-icons/fc";

import { env } from "@/src/env.mjs";
import Button from "@/src/ui/Button";
import DialogContent, { DialogRoot, DialogTrigger } from "@/src/ui/Dialog";

import RainbowkitWrapper from "./RainbowkitWrapper";

interface Props {
  loading?: boolean;
}

export default function SignInButton(props: Props) {
  return (
    <RainbowkitWrapper>
      <ClientSignInButton {...props} />
    </RainbowkitWrapper>
  );
}

export function ClientSignInButton({ loading }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <DialogRoot open={open} onOpenChange={setOpen}>
      <DialogContent title="Sign in to UNAVI">
        <SignInPage setOpen={setOpen} />
      </DialogContent>

      <DialogTrigger asChild>
        <button
          className={`rounded-full bg-neutral-900 px-6 py-1.5 font-bold text-white outline-neutral-400 transition active:scale-100 ${
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

export function SignInPage({ setOpen }: { setOpen?: (open: boolean) => void }) {
  const { openConnectModal } = useConnectModal();

  async function handleWalletLogin() {
    if (setOpen) setOpen(false);
    if (openConnectModal) openConnectModal();
  }

  return (
    <div className="flex flex-col items-center space-y-2">
      {env.NEXT_PUBLIC_HAS_GOOGLE_OAUTH ? (
        <div className="flex w-full items-center pt-4">
          <hr className="w-full border-neutral-300" />
          <span className="w-fit whitespace-nowrap px-4 font-bold text-neutral-700">
            Web3
          </span>
          <hr className="w-full border-neutral-300" />
        </div>
      ) : null}

      <Button onClick={handleWalletLogin}>Connect Wallet</Button>

      {env.NEXT_PUBLIC_HAS_GOOGLE_OAUTH ? (
        <>
          <div className="flex w-full items-center pt-8">
            <hr className="w-full border-neutral-300" />
            <span className="w-fit whitespace-nowrap px-4 font-bold text-neutral-700">
              Socials
            </span>
            <hr className="w-full border-neutral-300" />
          </div>
          <a
            href="/api/auth/methods/google"
            about="Sign in with Google"
            className="aspect-square rounded-full border border-neutral-400 p-2.5 text-2xl transition hover:bg-neutral-100"
          >
            <FcGoogle />
          </a>
        </>
      ) : null}
    </div>
  );
}
