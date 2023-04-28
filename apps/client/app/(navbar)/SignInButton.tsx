"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";
import { useRouter } from "next/navigation";
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

  const { openConnectModal } = useConnectModal();
  const router = useRouter();

  function handleWalletLogin() {
    setOpen(false);
    if (openConnectModal) openConnectModal();
  }

  function handleGoogleLogin() {
    // Open popup in center of screen
    const w = 500;
    const h = 600;
    const x = window.screenX + (window.innerWidth - w) / 2;
    const y = window.screenY + (window.innerHeight - h) / 2;

    const popup = window.open(
      "/api/auth/methods/google",
      "popup",
      `popup=true, width=${w}, height=${h}, left=${x}, top=${y}`
    );
    if (!popup) return;

    popup.focus();

    const interval = setInterval(() => {
      // Close popup when redirected back to our site
      if (popup.window.location.href.includes(env.NEXT_PUBLIC_DEPLOYED_URL)) {
        popup.close();
      }

      // Refresh page when popup is closed
      if (popup.closed) {
        clearInterval(interval);
        router.refresh();
        setOpen(false);
      }
    }, 50);
  }

  return (
    <DialogRoot open={open} onOpenChange={setOpen}>
      <DialogContent title="Sign in to UNAVI">
        <div className="flex flex-col items-center space-y-2">
          <div className="flex w-full items-center pt-4">
            <hr className="w-full border-neutral-300" />
            <span className="w-fit whitespace-nowrap px-4 font-bold text-neutral-700">Web3</span>
            <hr className="w-full border-neutral-300" />
          </div>

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
              <button
                onClick={handleGoogleLogin}
                className="aspect-square rounded-full border border-neutral-400 p-2.5 text-2xl transition hover:bg-neutral-100"
              >
                <FcGoogle />
              </button>
            </>
          ) : null}
        </div>
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
