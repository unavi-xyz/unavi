import Link from "next/link";
import { MdLogout, MdOutlinePersonOutline, MdOutlineSettings } from "react-icons/md";

import { useLogout } from "../../client/auth/useLogout";
import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import { DropdownItem } from "../../ui/DropdownMenu";
import { numberToHexDisplay } from "../../utils/numberToHexDisplay";

interface Props {
  includeExternal?: boolean;
}

export default function ProfileMenu({ includeExternal = true }: Props) {
  const { data: session } = useSession();
  const { logout } = useLogout();

  const { data: profile } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  return (
    <div className="py-2">
      {includeExternal && (
        <DropdownItem asChild>
          <Link
            href={`/user/${profile?.id ? numberToHexDisplay(profile.id) : session?.address}`}
            draggable={false}
            className="flex w-full cursor-pointer items-center whitespace-nowrap py-1 px-4 font-bold outline-none focus:bg-neutral-200 active:opacity-80"
          >
            <MdOutlinePersonOutline className="mr-2 text-lg" />
            <div>Your Profile</div>
          </Link>
        </DropdownItem>
      )}

      {includeExternal && (
        <DropdownItem asChild>
          <Link
            href="/settings"
            draggable={false}
            className="flex w-full cursor-pointer items-center whitespace-nowrap py-1 px-4 font-bold outline-none focus:bg-neutral-200 active:opacity-80"
          >
            <MdOutlineSettings className="mr-2 text-lg" />
            <div>Settings</div>
          </Link>
        </DropdownItem>
      )}

      <DropdownItem
        onClick={logout}
        className="flex w-full cursor-pointer items-center whitespace-nowrap py-1 px-4 font-bold outline-none focus:bg-neutral-200 active:opacity-80"
      >
        <MdLogout className="mr-2 text-lg" />
        <div>Log Out</div>
      </DropdownItem>
    </div>
  );
}
