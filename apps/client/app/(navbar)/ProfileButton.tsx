import { CustomSession } from "../../src/client/auth/useSession";
import Avatar from "../../src/home/Avatar";
import { fetchProfileFromAddress } from "../../src/server/helpers/fetchProfileFromAddress";
import { DropdownContent, DropdownMenu, DropdownTrigger } from "../../src/ui/DropdownMenu";
import ProfileMenu from "./ProfileMenu";

interface Props {
  session: CustomSession;
}

export default async function ProfileButton({ session }: Props) {
  const profile = session?.address ? await fetchProfileFromAddress(session.address) : null;

  return (
    <DropdownMenu>
      <DropdownTrigger className="rounded-full">
        <Avatar
          src={profile?.metadata?.image}
          uniqueKey={profile?.handle?.full ?? session?.address ?? ""}
          circle
          size={36}
        />
      </DropdownTrigger>

      <DropdownContent>
        <ProfileMenu profile={profile} session={session} />
      </DropdownContent>
    </DropdownMenu>
  );
}
