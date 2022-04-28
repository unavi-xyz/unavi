import { useState, useRef, useEffect } from "react";
import {
  MdOutlinePersonOutline,
  MdOutlineSettings,
  MdLogout,
} from "react-icons/md";
import Link from "next/link";

import { useLensStore } from "../../../helpers/lens/store";
import { logout } from "../../../helpers/lens/authentication";

import ProfileMenuButton from "./ProfileMenuButton";

interface Props {
  open: boolean;
}

export default function ProfileMenu({ open }: Props) {
  const menuRef = useRef<HTMLDivElement>(null);

  const handle = useLensStore((state) => state.handle);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const timeout = setTimeout(() => setVisible(false), 150);

    if (open) {
      setVisible(true);
      clearTimeout(timeout);
    }

    if (open) {
      setTimeout(() => {
        menuRef.current?.classList.remove("scale-75");
        menuRef.current?.classList.remove("opacity-0");
      });
    } else {
      menuRef.current?.classList.add("opacity-0");
      menuRef.current?.classList.add("scale-75");
    }

    return () => clearTimeout(timeout);
  }, [open]);

  if (!visible) return null;

  return (
    <div
      ref={menuRef}
      className="w-48 bg-white border py-2 rounded-lg space-y-2
                 transition-all duration-150 ease-in-out scale-75 opacity-0"
    >
      <div className="px-2">
        <Link href={`/user/${handle}`} passHref>
          <div>
            <ProfileMenuButton>
              <div className="gradient-text">@{handle}</div>
            </ProfileMenuButton>
          </div>
        </Link>
      </div>

      <hr />

      <div className="px-2 space-y-2">
        <Link href={`/user/${handle}`} passHref>
          <div>
            <ProfileMenuButton icon={<MdOutlinePersonOutline />}>
              Your Profile
            </ProfileMenuButton>
          </div>
        </Link>

        <Link href="/settings" passHref>
          <div>
            <ProfileMenuButton icon={<MdOutlineSettings />}>
              Settings
            </ProfileMenuButton>
          </div>
        </Link>

        <div onClick={logout}>
          <ProfileMenuButton icon={<MdLogout />}>Log Out </ProfileMenuButton>
        </div>
      </div>
    </div>
  );
}
