import Head from "next/head";
import Link from "next/link";
import { FaHashtag, FaTwitter } from "react-icons/fa";
import { MdOutlineLocationOn } from "react-icons/md";

import { useMediaImage } from "../../../helpers/lens/hooks/useMediaImage";
import { useLensStore } from "../../../helpers/lens/store";
import Button from "../../base/Button";
import NavigationTab from "../../base/NavigationTab";
import ProfilePicture from "../../lens/ProfilePicture";
import MetaTags from "../../ui/MetaTags";
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
}: Props) {
  const coverUrl = useMediaImage(profile?.coverPicture);
  const viewerHandle = useLensStore((state) => state.handle);

  const twitter = profile?.attributes?.find((item) => item.key === "twitter");
  const website = profile?.attributes?.find((item) => item.key === "website");
  const location = profile?.attributes?.find((item) => item.key === "location");

  return (
    <>
      <MetaTags
        title={metadata.title}
        description={metadata.description}
        image={metadata.image}
        imageWidth="256px"
        imageHeight="256px"
      />

      <Head>
        <meta property="og:type" content="profile" />
        <meta property="og:profile:username" content={handle} />
        <meta
          property="og:profile:first_name"
          content={profile?.name ?? handle}
        />
      </Head>

      {profile && (
        <div>
          <div className="w-full h-80 bg-tertiaryContainer">
            {coverUrl && (
              <img
                src={coverUrl}
                alt="cover"
                className="w-full h-full object-cover"
              />
            )}
          </div>

          <div className="flex justify-center">
            <div className="max-w mx-4 pb-4 flex flex-col md:flex-row">
              <div className="md:w-64 flex-shrink-0 space-y-6">
                <div className="h-32 w-64">
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
                      <a>
                        <Button variant="outlined" fullWidth>
                          Edit Profile
                        </Button>
                      </a>
                    </Link>
                  )}
                </div>

                <div className="font-bold">{profile?.bio}</div>

                <div className="space-y-4">
                  <AttributeRow icon={<FaHashtag className="text-lg" />}>
                    {profile.id}
                  </AttributeRow>

                  {location && (
                    <AttributeRow icon={<MdOutlineLocationOn />}>
                      {location.value}
                    </AttributeRow>
                  )}

                  {twitter && (
                    <AttributeRow icon={<FaTwitter className="text-sky-500" />}>
                      <a
                        href={`https://twitter.com/${twitter.value}`}
                        target="_blank"
                        rel="noreferrer"
                        className="hover:underline"
                      >
                        {twitter.value}
                      </a>
                    </AttributeRow>
                  )}

                  {website && (
                    <AttributeRow
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
                    </AttributeRow>
                  )}
                </div>
              </div>

              <div className="w-full space-y-4 md:ml-12 pt-4">
                <div className="flex items-center justify-center w-full space-x-4">
                  <NavigationTab text="Spaces" href={`/user/${handle}`} />
                  <NavigationTab
                    text="Avatars"
                    href={`/user/${handle}/avatars`}
                  />
                </div>

                <div>{children}</div>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
