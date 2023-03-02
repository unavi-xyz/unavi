"use client";

import { useRouter } from "next/navigation";
import { SessionProvider } from "next-auth/react";
import { useEffect, useTransition } from "react";

import { useSession } from "../../src/client/auth/useSession";
import ProfileButton from "./ProfileButton";
import SignInButton from "./SignInButton";

export default function ClientButtons() {
  return (
    <SessionProvider>
      <Buttons />
    </SessionProvider>
  );
}

function Buttons() {
  const { data: session, status } = useSession();
  const router = useRouter();
  const [transitionLoading, startTransition] = useTransition();

  useEffect(() => {
    if (status === "loading") return;

    startTransition(() => {
      router.refresh();
    });
  }, [status, router, startTransition]);

  const isLoading = transitionLoading || status === "loading";

  return session?.address ? (
    <ProfileButton isLoading={isLoading} />
  ) : (
    <SignInButton isLoading={isLoading} />
  );
}
