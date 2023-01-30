import { useConnectModal } from "@rainbow-me/rainbowkit";

export default function SignInButton() {
  const { openConnectModal } = useConnectModal();

  return (
    <button
      className="rounded-full bg-neutral-900 px-7 py-1.5 text-lg font-bold text-white transition hover:scale-105"
      onClick={openConnectModal}
    >
      Sign in
    </button>
  );
}
