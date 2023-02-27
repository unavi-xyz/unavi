"use client";

import "@rainbow-me/rainbowkit/styles.css";

import { RainbowKitProvider, useConnectModal } from "@rainbow-me/rainbowkit";
import {
  GetSiweMessageOptions,
  RainbowKitSiweNextAuthProvider,
} from "@rainbow-me/rainbowkit-siwe-next-auth";
import { useRouter } from "next/navigation";
import { SessionProvider } from "next-auth/react";
import { useEffect, useTransition } from "react";
import { WagmiConfig } from "wagmi";

import { useSession } from "../../src/client/auth/useSession";
import { theme } from "../../src/client/rainbow";
import { chains, wagmiClient } from "../../src/client/wagmi";

const getSiweMessageOptions: GetSiweMessageOptions = () => ({
  statement: "🔌 Sign in to the Wired",
});

export default function SignInButton() {
  return (
    <SessionProvider>
      <WagmiConfig client={wagmiClient}>
        <RainbowKitSiweNextAuthProvider getSiweMessageOptions={getSiweMessageOptions}>
          <RainbowKitProvider theme={theme} chains={chains}>
            <Button />
          </RainbowKitProvider>
        </RainbowKitSiweNextAuthProvider>
      </WagmiConfig>
    </SessionProvider>
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
