"use client";

import { useAuth } from "@/src/client/AuthProvider";
import { useProfile } from "@/src/play/hooks/useProfile";

import ProfileButton from "./ProfileButton";
import SignInButton from "./SignInButton";

export default function AccountButton() {
  const { status, loading: transitionLoading, user } = useAuth();
  const { profile, loading: profileLoading } = useProfile();

  const loading = status === "loading" || transitionLoading || profileLoading;

  return user ? (
    <ProfileButton user={user} image={profile?.image} loading={loading} />
  ) : (
    <SignInButton loading={loading} />
  );
}
