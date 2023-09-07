"use client";

import { useAuth } from "@/src/client/AuthProvider";
import { useProfile } from "@/src/play/hooks/useProfile";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import ProfileButton from "./ProfileButton";
import SignInButton from "./SignInButton";

export default function AccountButton() {
  const { status, loading: transitionLoading, user } = useAuth();
  const { profile, loading: profileLoading } = useProfile();

  const loading = status === "loading" || transitionLoading || profileLoading;

  const image =
    user && profile?.image
      ? cdnURL(S3Path.profile(user.userId).image(profile.image))
      : undefined;

  return user ? (
    <ProfileButton
      did={user.did}
      username={user.username}
      image={image}
      loading={loading}
    />
  ) : (
    <SignInButton loading={loading} />
  );
}
