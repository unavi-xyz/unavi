"use client";

import AuthProvider, { useAuth } from "@/src/client/AuthProvider";
import { useProfile } from "@/src/play/hooks/useProfile";

import ProfileButton from "./ProfileButton";
import SignInButton from "./SignInButton";

export default async function AccountButton() {
  return (
    <AuthProvider>
      <ClientAccountButton />
    </AuthProvider>
  );
}

function ClientAccountButton() {
  const { user } = useAuth();
  const { profile } = useProfile();

  return user ? <ProfileButton user={user} image={profile?.image} /> : <SignInButton />;
}
