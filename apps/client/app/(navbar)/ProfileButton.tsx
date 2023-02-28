"use client";

import useSWR from "swr";

import { CustomSession } from "../../src/client/auth/useSession";
import Avatar from "../../src/home/Avatar";
import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../src/ui/DropdownMenu";
import { getProfileByAddress } from "../api/profile/address/[address]/helper";
import ProfileMenu from "./ProfileMenu";

interface Props {
  session: CustomSession;
}

export default function ProfileButton({ session }: Props) {
  if (!session.address) throw new Error("No address found");

  const { data, isLoading } = useSWR(session.address, getProfileByAddress);

  return (
    <DropdownMenu>
      <DropdownTrigger className="rounded-full">
        <Avatar
          src={data?.metadata?.image}
          uniqueKey={data?.handle?.full ?? session?.address ?? ""}
          loading={isLoading}
          circle
          size={36}
        />
      </DropdownTrigger>

      <DropdownContent>
        {data !== undefined && <ProfileMenu profile={data} session={session} />}
      </DropdownContent>
    </DropdownMenu>
  );
}
