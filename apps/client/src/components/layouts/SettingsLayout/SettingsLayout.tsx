import {
  MdOutlinePersonOutline,
  MdOutlineAccountBalanceWallet,
  MdOutlineWarningAmber,
} from "react-icons/md";
import Link from "next/link";
import Head from "next/head";
import { useRouter } from "next/router";

import { useLensStore } from "../../../helpers/lens/store";
import { useProfileByHandle } from "../../../helpers/lens/hooks/useProfileByHandle";

import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import SettingsButton from "./SettingsButton";
import NavbarLayout from "../NavbarLayout/NavbarLayout";

interface Props {
  children: React.ReactNode;
}

export default function SettingsLayout({ children }: Props) {
  const router = useRouter();

  const handle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);

  if (!handle) return null;

  return (
    <div className="flex justify-center">
      <Head>
        <title>Settings · The Wired</title>
      </Head>

      <div className="max-w mx-8 mt-8 flex">
        <div className="max-w-md w-full p-8 space-y-4">
          <Link href={`/user/${handle}`} passHref>
            <div className="flex space-x-4 cursor-pointer">
              <div className="w-14 h-14 rounded-full border">
                <ViewerProfilePicture circle />
              </div>
              <div>
                <div className="font-bold text-lg">
                  {profile?.name ?? handle}
                </div>
                <div className="gradient-text w-min">@{handle}</div>
              </div>
            </div>
          </Link>

          <Link href="/settings" passHref>
            <div>
              <SettingsButton
                icon={<MdOutlinePersonOutline />}
                selected={router.asPath === "/settings"}
              >
                Profile
              </SettingsButton>
            </div>
          </Link>

          <Link href="/settings/account" passHref>
            <div>
              <SettingsButton
                icon={<MdOutlineAccountBalanceWallet />}
                selected={router.asPath === "/settings/account"}
              >
                Account
              </SettingsButton>
            </div>
          </Link>

          <Link href="/settings/delete" passHref>
            <div className="text-red-500">
              <SettingsButton
                icon={<MdOutlineWarningAmber />}
                selected={router.asPath === "/settings/delete"}
              >
                Danger Zone
              </SettingsButton>
            </div>
          </Link>
        </div>

        <div className="w-full">{children}</div>
      </div>
    </div>
  );
}

SettingsLayout.Layout = NavbarLayout;
