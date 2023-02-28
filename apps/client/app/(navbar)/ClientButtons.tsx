"use client";

import { SessionProvider } from "next-auth/react";

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
  const { data: session } = useSession();
  return session && session.address ? <ProfileButton session={session} /> : <SignInButton />;
}
