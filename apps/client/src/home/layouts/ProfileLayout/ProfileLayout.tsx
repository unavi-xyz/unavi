import Image from "next/future/image";
import Head from "next/head";
import Link from "next/link";
import { useContext } from "react";
import { FaTwitter } from "react-icons/fa";
import { MdAdd, MdLink, MdOutlineLocationOn } from "react-icons/md";

import { LensContext } from "@wired-xr/lens";

import MetaTags from "../../../ui/MetaTags";
import Button from "../../../ui/base/Button";
import ProfilePicture from "../../lens/ProfilePicture";
import AttributeRow from "./AttributeRow";
import { ProfileLayoutProps } from "./getProfileLayoutProps";

interface Props extends ProfileLayoutProps {
  children: React.ReactNode;
}

export default function ProfileLayout({
  children,
  handle,
  metadata,
  profile,
  coverImage,
  profileImage,
}: Props) {
  const { handle: viewerHandle } = useContext(LensContext);

  const twitter = profile?.attributes?.find((item) => item.key === "twitter");
  const website = profile?.attributes?.find((item) => item.key === "website");
  const location = profile?.attributes?.find((item) => item.key === "location");

  return (
    <>
      <MetaTags
        title={metadata.title}
        description={metadata.description}
        image={metadata.image}
      />

      <Head>
        <meta property="og:type" content="profile" />
        <meta property="og:profile:username" content={handle} />
        <meta
          property="og:profile:first_name"
          content={profile?.name ?? handle}
        />
      </Head>

      {profile ? (
        <div className="max-w mx-auto">
          <div className="w-full h-48 md:h-64 bg-primaryContainer md:rounded-3xl">
            <div className="relative object-cover w-full h-full">
              {coverImage && (
                <Image
                  src={coverImage}
                  priority
                  fill
                  sizes="80vw"
                  alt="cover"
                  className="w-full h-full object-cover md:rounded-3xl"
                />
              )}
            </div>
          </div>

          <div className="flex justify-center pb-4 px-4 md:px-0">
            <div className="w-full flex flex-col items-center space-y-2">
              <div className="w-32 rounded-full -mt-16 flex z-10 ring-4 ring-background">
                {
                  <ProfilePicture
                    src={profileImage ?? `https://avatar.tobi.sh/${handle}`}
                    circle
                  />
                }
              </div>

              <div className="flex flex-col items-center">
                <div className="text-2xl font-black">{handle}</div>
                {/* <div className="text-lg font-bold">{profile.name}</div> */}
              </div>

              <div className="w-full py-2 flex space-x-4 justify-center">
                <div className="flex flex-col md:flex-row md:space-x-1 items-center">
                  <div className="font-black text-lg">
                    {profile.stats.totalFollowing}
                  </div>
                  <div className="text-outline leading-5 text-lg">
                    Following
                  </div>
                </div>

                <div className="flex flex-col md:flex-row md:space-x-1 items-center">
                  <div className="font-black text-lg">
                    {profile.stats.totalFollowers}
                  </div>
                  <div className="text-outline leading-5 text-lg">
                    Followers
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div className="w-full flex flex-col items-center md:items-start px-4 md:px-0">
            <div className="w-full space-y-2 flex flex-col items-center">
              <div className="w-full flex justify-center space-x-2">
                {handle === viewerHandle ? (
                  <Link href="/settings" passHref>
                    <a>
                      <Button variant="outlined" squared="small">
                        <div className="px-6">Edit profile</div>
                      </Button>
                    </a>
                  </Link>
                ) : (
                  <div>
                    <Button
                      variant="filled"
                      squared="small"
                      disabled={!viewerHandle}
                    >
                      <div className="flex justify-center items-center space-x-1 px-6">
                        <MdAdd />
                        <div>Follow</div>
                      </div>
                    </Button>
                  </div>
                )}

                {twitter && (
                  <Button variant="outlined" squared="small">
                    <a
                      href={`https://twitter.com/${twitter.value}`}
                      target="_blank"
                      rel="noreferrer"
                      className="hover:underline"
                    >
                      <FaTwitter className="text-lg" />
                    </a>
                  </Button>
                )}
              </div>

              <div className="w-full pt-2">
                <div className="text-sm md:text-base text-center">
                  {profile.bio}
                </div>
              </div>

              <div className="flex space-x-4 flex-wrap">
                {location && (
                  <AttributeRow icon={<MdOutlineLocationOn />}>
                    {location.value}
                  </AttributeRow>
                )}

                {website && (
                  <AttributeRow icon={<MdLink />}>
                    <a
                      href={website.value}
                      target="_blank"
                      rel="noreferrer"
                      className="hover:underline"
                    >
                      {website.value}
                    </a>
                  </AttributeRow>
                )}
              </div>
            </div>

            <div className="w-full md:mt-4">{children}</div>
          </div>
        </div>
      ) : (
        <div className="flex justify-center text-lg pt-12">User not found.</div>
      )}
    </>
  );
}
