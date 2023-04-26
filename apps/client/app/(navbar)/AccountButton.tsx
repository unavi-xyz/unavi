"use client";

import AuthProvider, { useAuth } from "@/src/client/AuthProvider";

import ProfileButton from "./ProfileButton";
import SignInButton from "./SignInButton";

export default function AccountButton() {
  return (
    <AuthProvider>
      <Buttons />
    </AuthProvider>
  );
}

function Buttons() {
  const { status, loading } = useAuth();

  return status === "authenticated" ? (
    <ProfileButton loading={loading} />
  ) : (
    <SignInButton loading={loading || status === "loading"} />
  );
}
