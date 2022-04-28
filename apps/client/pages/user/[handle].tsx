import Link from "next/link";
import { useRouter } from "next/router";

import { useLensStore } from "../../src/helpers/lens/store";
import { useProfileByHandle } from "../../src/helpers/lens/hooks/useProfileByHandle";
import { useColorFromSeed } from "../../src/helpers/hooks/useColorFromSeed";

import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";
import ProfilePicture from "../../src/components/lens/ProfilePicture";
import Button from "../../src/components/base/Button";

export default function User() {
  const router = useRouter();
  const handle = router.query.handle as string;

  const viewerHandle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);
  const color = useColorFromSeed(profile?.handle);

  return (
    <div>
      <div style={{ backgroundColor: color }} className="w-full h-72" />

      <div className="flex justify-center">
        <div className="max-w mx-8">
          <div className="w-72 space-y-8">
            <div className="h-32">
              <div className="relative w-full">
                <div
                  className="absolute bottom-0 transform translate-y-1/2
                             h-72 w-72 rounded-xl ring-8 ring-neutral-100"
                >
                  <ProfilePicture profile={profile} />
                </div>
              </div>
            </div>

            <div className="space-y-2">
              <div className="text-2xl font-black">
                {profile?.name ?? handle}
              </div>
              <div className="gradient-text text-lg w-min">@{handle}</div>
            </div>

            <div className="flex justify-between">
              <div className="flex space-x-2 text-lg">
                <div className="font-black">0</div>
                <div className="font-bold text-neutral-500">Following</div>
              </div>

              <div className="flex space-x-2 text-lg">
                <div className="font-black">0</div>
                <div className="font-bold text-neutral-500">Followers</div>
              </div>
            </div>

            <div>
              {handle === viewerHandle ? (
                <Link href="/settings" passHref>
                  <div>
                    <Button outline>Edit Profile</Button>
                  </div>
                </Link>
              ) : (
                <Button outline>Follow</Button>
              )}
            </div>

            <div className="font-bold">{profile?.bio}</div>
          </div>
        </div>
      </div>
    </div>
  );
}

User.Layout = NavbarLayout;
