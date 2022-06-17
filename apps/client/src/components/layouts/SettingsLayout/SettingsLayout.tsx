import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";
import {
  MdAdd,
  MdOutlineAccountBalanceWallet,
  MdOutlinePersonOutline,
} from "react-icons/md";

import { useProfileByHandle } from "../../../helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../../helpers/lens/store";
import Dialog from "../../base/Dialog";
import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import CreateProfilePage from "../NavbarLayout/CreateProfilePage";
import { getNavbarLayout } from "../NavbarLayout/NavbarLayout";
import SettingsButton from "./SettingsButton";

interface Props {
  children: React.ReactNode;
}

export default function SettingsLayout({ children }: Props) {
  const router = useRouter();
  const handle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);

  const [open, setOpen] = useState(false);

  if (!handle) return null;

  return (
    <div className="flex justify-center">
      <Head>
        <title>Settings / The Wired</title>
      </Head>

      <Dialog open={open} onClose={() => setOpen(false)}>
        <CreateProfilePage />
      </Dialog>

      <div className="max-w mx-4 mb-4 flex flex-col md:flex-row">
        <div className="pt-8 md:pr-8 space-y-2 w-full md:max-w-xs">
          <div className="flex space-x-4 pb-4">
            <div className="w-20 flex flex-col justify-center">
              <ViewerProfilePicture circle />
            </div>
            <div className="flex flex-col justify-center">
              <div className="font-black text-lg break-all">
                {profile?.name ?? handle}
              </div>
              <div className="gradient-text break-all font-bold">@{handle}</div>
            </div>
          </div>

          <Link href="/settings" passHref>
            <a className="block">
              <SettingsButton
                icon={<MdOutlinePersonOutline />}
                selected={router.asPath === "/settings"}
              >
                Profile
              </SettingsButton>
            </a>
          </Link>

          <Link href="/settings/account" passHref>
            <a className="block">
              <SettingsButton
                icon={<MdOutlineAccountBalanceWallet />}
                selected={router.asPath === "/settings/account"}
              >
                Account
              </SettingsButton>
            </a>
          </Link>

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
