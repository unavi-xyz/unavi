import Head from "next/head";
import Link from "next/link";
import { FaHashtag, FaTwitter } from "react-icons/fa";
import { MdOutlineLocationOn } from "react-icons/md";

import { useAvatarUrlFromProfile } from "../../../helpers/lens/hooks/useAvatarFromProfile";
import { useMediaImage } from "../../../helpers/lens/hooks/useMediaImage";
import { useLensStore } from "../../../helpers/lens/store";
import { DEFAULT_AVATAR_URL } from "../../app/OtherPlayer";
import Button from "../../base/Button";
import NavigationTab from "../../base/NavigationTab";
import ProfilePicture from "../../lens/ProfilePicture";
import MetaTags from "../../ui/MetaTags";
import AvatarCanvas from "../AvatarLayout/AvatarCanvas";
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
  const avatarUrl = useAvatarUrlFromProfile(profile);

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

      <div className="w-full h-80 bg-tertiaryContainer">
        {coverUrl && (
          <img
            src={coverUrl}
            alt="cover"
            className="w-full h-full object-cover"
          />
        )}
      </div>

      {profile && (
        <div className="max-w mx-auto">
          <div className="w-full flex flex-col items-center md:items-start md:flex-row p-4">
            <div className="w-full md:w-1/3 space-y-4">
              <div className="aspect-vertical rounded-3xl bg-cover -mt-48 md:-mt-72 w-1/2 md:w-full mx-auto md:mx-0">
                <AvatarCanvas
                  url={avatarUrl ?? DEFAULT_AVATAR_URL}
                  background={false}
                />
              </div>

              <div className="p-2 flex items-center space-x-4">
                <div className="w-1/3 rounded-full ring-background">
                  <ProfilePicture profile={profile} circle />
                </div>

                <div className="w-full">
                  <div className="text-2xl font-black">
                    {profile.name ?? handle}
                  </div>
                  <div className="text-lg font-bold gradient-text">
                    @{handle}
                  </div>
                </div>
              </div>

              <div className="p-2 font-bold">{profile?.bio}</div>

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

              <div className="p-2 space-y-4">
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

            <div className="md:p-4 pt-4 w-full space-y-4 md:ml-12">
              <div>{children}</div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
