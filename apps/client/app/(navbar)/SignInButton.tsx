"use client";

import { useConnectModal } from "@rainbow-me/rainbowkit";

import RainbowkitWrapper from "./RainbowkitWrapper";

interface Props {
  isLoading?: boolean;
}

export default function SignInButton(props: Props) {
  return (
    <RainbowkitWrapper>
      <Button {...props} />
    </RainbowkitWrapper>
  );
}

function Button({ isLoading }: Props) {
  const { openConnectModal } = useConnectModal();

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
