import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import {
  MdOutlineAccountBalanceWallet,
  MdOutlinePersonOutline,
} from "react-icons/md";

import { useProfileByHandle } from "../../../helpers/lens/hooks/useProfileByHandle";
import { useLensStore } from "../../../helpers/lens/store";
import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import NavbarLayout, { getNavbarLayout } from "../NavbarLayout/NavbarLayout";
import SettingsButton from "./SettingsButton";

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

      <div className="max-w mx-8 my-8 flex">
        <div className="pt-8 pr-8 space-y-4 w-full max-w-xs">
          <div className="flex space-x-4">
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
        </div>

        <div className="w-full">{children}</div>
      </div>
    </div>
  );
}

export function getSettingsLayout(children: React.ReactNode) {
  return getNavbarLayout(<SettingsLayout>{children}</SettingsLayout>);
}
