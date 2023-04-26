"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";

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
  const { openConnectModal } = useConnectModal();

  return (
    <button
      className={`rounded-full bg-neutral-900 px-6 py-1.5 font-bold text-white outline-neutral-400 transition active:scale-100 ${
        loading ? "opacity-70" : "hover:scale-105"
      }`}
      disabled={loading}
      onClick={openConnectModal}
    >
      Sign in
    </button>
  );
}
