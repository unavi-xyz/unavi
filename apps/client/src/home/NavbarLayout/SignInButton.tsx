import { useConnectModal } from "@rainbow-me/rainbowkit";

export default function SignInButton() {
  const { openConnectModal } = useConnectModal();

  return (
    <button
      className="rounded-full bg-neutral-900 px-6 py-1.5 font-bold text-white outline-neutral-400 transition hover:scale-105 active:scale-100"
      onClick={openConnectModal}
    >
      Sign in
    </button>
  );
}
