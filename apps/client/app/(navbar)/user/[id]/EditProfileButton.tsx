"use client";

import Link from "next/link";
import { SessionProvider } from "next-auth/react";

import { useSession } from "../../../../src/client/auth/useSession";
import { Profile } from "../../../../src/server/helpers/fetchProfile";

interface Props {
  id: string;
  profile: Profile | null;
}

export default function EditProfileButton(props: Props) {
  return (
    <SessionProvider>
      <Button {...props} />
    </SessionProvider>
  );
}

function Button({ id, profile }: Props) {
  const isAddress = id.length === 42;

  const { data: session } = useSession();

  const isUser = !session?.address
    ? false
    : isAddress
    ? session.address === id
    : session.address === profile?.owner;

  if (!isUser) return null;

  return (
    <div className="flex w-full justify-center space-x-2">
      <Link
        href="/settings"
        className="rounded-md px-10 py-1.5 font-bold ring-1 ring-neutral-700 transition hover:bg-neutral-200 active:bg-neutral-300"
      >
        Edit profile
      </Link>
    </div>
  );
}
