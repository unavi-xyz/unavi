import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";
import {
  MdAdd,
  MdOutlineAccountBalanceWallet,
  MdOutlinePersonOutline,
} from "react-icons/md";

import { useLens } from "../../../client/lens/hooks/useLens";
import { useProfileByHandle } from "../../../client/lens/hooks/useProfileByHandle";
import Dialog from "../../../ui/Dialog";
import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import CreateProfilePage from "../NavbarLayout/CreateProfilePage";
import { getNavbarLayout } from "../NavbarLayout/NavbarLayout";
import SettingsButton from "./SettingsButton";

interface Props {
  children: React.ReactNode;
}

export default function SettingsLayout({ children }: Props) {
  const router = useRouter();
  const { handle } = useLens();
  const profile = useProfileByHandle(handle);

  const [open, setOpen] = useState(false);

  if (!handle || !profile) return null;

  return (
    <div className="flex justify-center">
      <Dialog open={open} onClose={() => setOpen(false)}>
        <CreateProfilePage />
      </Dialog>

      <div className="max-w-content mx-4 mb-4 flex flex-col md:flex-row">
        <div className="w-full space-y-2 pt-8 md:max-w-xs md:pr-8">
          <div className="flex space-x-4 pb-4">
            <div className="flex w-20 flex-col justify-center p-1">
              <ViewerProfilePicture circle size={80} />
            </div>
            <div className="flex flex-col justify-center">
              <div className="break-all text-lg font-black ">@{handle}</div>
              <div className="break-all font-bold">{profile.name}</div>
            </div>
          </div>

          <div>
            <Link href="/settings">
              <SettingsButton
                icon={<MdOutlinePersonOutline />}
                selected={router.asPath === "/settings"}
              >
                Profile
              </SettingsButton>
            </Link>
          </div>

          <div>
            <Link href="/settings/account">
              <SettingsButton
                icon={<MdOutlineAccountBalanceWallet />}
                selected={router.asPath === "/settings/account"}
              >
                Account
              </SettingsButton>
            </Link>
          </div>

          <SettingsButton icon={<MdAdd />} onClick={() => setOpen(true)}>
            Create Profile
          </SettingsButton>
        </div>

        <div className="w-full pt-4">{children}</div>
      </div>
    </div>
  );
}

export function getSettingsLayout(children: React.ReactNode) {
  return getNavbarLayout(<SettingsLayout>{children}</SettingsLayout>);
}
