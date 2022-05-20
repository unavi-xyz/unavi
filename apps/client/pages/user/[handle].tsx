import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import { FaHashtag, FaTwitter } from "react-icons/fa";
import { MdOutlineLocationOn } from "react-icons/md";

import Button from "../../src/components/base/Button";
import NavbarLayout from "../../src/components/layouts/NavbarLayout/NavbarLayout";
import ProfilePicture from "../../src/components/lens/ProfilePicture";
import SpaceCard from "../../src/components/ui/SpaceCard";
import { useMediaImage } from "../../src/helpers/lens/hooks/useMediaImage";
import { useProfileByHandle } from "../../src/helpers/lens/hooks/useProfileByHandle";
import { useSpacesByProfile } from "../../src/helpers/lens/hooks/useSpacesByProfile";
import { useLensStore } from "../../src/helpers/lens/store";

export default function User() {
  const router = useRouter();
  const handle = router.query.handle as string;

  const profile = useProfileByHandle(handle);
  const coverUrl = useMediaImage(profile?.coverPicture);
  const viewerHandle = useLensStore((state) => state.handle);
  const spaces = useSpacesByProfile(profile?.id);

  if (!profile) return null;

  const twitter = profile.attributes?.find((item) => item.key === "twitter");
  const website = profile.attributes?.find((item) => item.key === "website");
  const location = profile.attributes?.find((item) => item.key === "location");

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
        }}
        className="w-full h-80 bg-cover bg-center bg-tertiaryContainer"
      />

      <div className="flex justify-center">
        <div className="max-w mx-8 flex space-x-12">
          <div className="w-64 flex-shrink-0 space-y-6">
            <div className="h-32">
              <div className="relative w-full">
                <div
                  className="absolute -bottom-32 transform
                             rounded-xl ring-8 ring-background"
                >
                  <ProfilePicture profile={profile} />
                </div>
              </div>
            </div>

            <div className="space-y-2">
              <div className="text-2xl font-black break-all">
                {profile?.name ?? handle}
              </div>
              <div className="gradient-text text-lg break-all font-bold w-fit">
                @{handle}
              </div>
            </div>

            <div>
              {handle === viewerHandle && (
                <Link href="/settings" passHref>
                  <div>
                    <Button variant="outlined" fullWidth>
                      Edit Profile
                    </Button>
                  </div>
                </Link>
              )}
            </div>

            <div className="font-bold">{profile?.bio}</div>

            <div className="space-y-4">
              <ProfileRow icon={<FaHashtag className="text-lg" />}>
                {profile.id}
              </ProfileRow>

              {location && (
                <ProfileRow icon={<MdOutlineLocationOn />}>
                  {location.value}
                </ProfileRow>
              )}

              {twitter && (
                <ProfileRow icon={<FaTwitter className="text-sky-500" />}>
                  <a
                    href={`https://twitter.com/${twitter.value}`}
                    target="_blank"
                    rel="noreferrer"
                    className="hover:underline"
                  >
                    {twitter.value}
                  </a>
                </ProfileRow>
              )}

              {website && (
                <ProfileRow
                  icon={
                    <img
                      src={`https://s2.googleusercontent.com/s2/favicons?domain_url=${website.value}`}
                      alt="website favicon"
                    />
                  }
                >
                  <a
                    href={website.value}
                    target="_blank"
                    rel="noreferrer"
                    className="hover:underline"
                  >
                    {website.value}
                  </a>
                </ProfileRow>
              )}
            </div>
          </div>

          {spaces && spaces.length > 1 && (
            <div className="w-full p-4 space-y-4">
              <div className="flex items-center justify-center w-full space-x-4">
                <div className="text-xl font-bold rounded-lg px-3 py-1">
                  Spaces
                </div>
              </div>

              <div className="space-y-2">
                {spaces?.map((space) => (
                  <div key={space.id}>
                    <Link href={`/space/${space.id}`} passHref>
                      <div>
                        <SpaceCard space={space} />
                      </div>
                    </Link>
                  </div>
                ))}
              </div>
            </div>
          )}
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
