import { FaHashtag, FaTwitter } from "react-icons/fa";
import { MdOutlineLocationOn } from "react-icons/md";
import Link from "next/link";
import Head from "next/head";
import { useRouter } from "next/router";

import { useLensStore } from "../../src/helpers/lens/store";
import { useProfileByHandle } from "../../src/helpers/lens/hooks/useProfileByHandle";
import { useColorFromSeed } from "../../src/helpers/hooks/useColorFromSeed";
import { useMediaImage } from "../../src/helpers/lens/hooks/useMediaImage";

import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";
import ProfilePicture from "../../src/components/lens/ProfilePicture";
import Button from "../../src/components/base/Button";

export default function User() {
  const router = useRouter();
  const handle = router.query.handle as string;

  const viewerHandle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);
  const color = useColorFromSeed(profile?.handle);
  const { url: coverUrl } = useMediaImage(profile?.coverPicture);

  if (!profile) return null;

  return (
    <div>
      <Head>
        {profile?.name ? (
          <title>
            {profile.name} (@{profile.handle}) · The Wired
          </title>
        ) : (
          <title>@{profile.handle} · The Wired</title>
        )}
      </Head>

      <div
        style={{
          backgroundImage: coverUrl ? `url(${coverUrl})` : undefined,
          backgroundColor: color,
        }}
        className="w-full h-80 bg-cover bg-center"
      />

      <div className="flex justify-center">
        <div className="max-w mx-8 flex space-x-12">
          <div className="w-64 flex-shrink-0 space-y-6">
            <div className="h-32">
              <div className="relative w-full">
                <div
                  className="absolute -bottom-32 transform
                             rounded-xl ring-8 ring-neutral-100"
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

            <div className="space-y-4">
              <ProfileRow icon={<FaHashtag className="text-lg" />}>
                {profile.id}
              </ProfileRow>

              {profile?.location && (
                <ProfileRow icon={<MdOutlineLocationOn />}>
                  {profile?.location}
                </ProfileRow>
              )}

              {profile?.twitter && (
                <ProfileRow icon={<FaTwitter className="text-sky-500" />}>
                  <a
                    href={`https://twitter.com/${profile.twitter}`}
                    target="_blank"
                    rel="noreferrer"
                  >
                    {profile.twitter}
                  </a>
                </ProfileRow>
              )}

              {profile?.website && (
                <ProfileRow
                  icon={
                    <img
                      src={`https://s2.googleusercontent.com/s2/favicons?domain_url=${profile.website}`}
                      alt="website favicon"
                    />
                  }
                >
                  <a href={profile.website} target="_blank" rel="noreferrer">
                    {profile.website}
                  </a>
                </ProfileRow>
              )}
            </div>
          </div>

          <div className="w-full p-4 space-y-4">
            <div className="flex items-center justify-center w-full space-x-4">
              <div className="font-bold bg-neutral-200 rounded-lg px-3 py-1 cursor-pointer">
                Spaces
              </div>
              <div className="font-bold hover:bg-neutral-200 rounded-lg px-3 py-1 cursor-pointer">
                Mirrors
              </div>
              <div className="font-bold hover:bg-neutral-200 rounded-lg px-3 py-1 cursor-pointer">
                Avatars
              </div>
            </div>

            <div className="h-24 bg-white rounded-xl border"></div>
          </div>
        </div>
      </div>
    </div>
  );
}

User.Layout = NavbarLayout;

function ProfileRow({
  icon,
  children,
}: {
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center space-x-6">
      <div className="text-xl w-6">{icon}</div>
      <div className="font-bold">{children}</div>
    </div>
  );
}
