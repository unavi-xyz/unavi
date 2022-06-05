import { NextPageContext } from "next";
import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import { FaHashtag, FaTwitter } from "react-icons/fa";
import { MdOutlineLocationOn } from "react-icons/md";

import Button from "../../src/components/base/Button";
import { getNavbarLayout } from "../../src/components/layouts/NavbarLayout/NavbarLayout";
import ProfilePicture from "../../src/components/lens/ProfilePicture";
import SpaceCard from "../../src/components/lens/SpaceCard";
import MetaTags from "../../src/components/ui/MetaTags";
import {
  GetProfileByHandleDocument,
  GetProfileByHandleQuery,
  GetProfileByHandleQueryVariables,
  GetPublicationsDocument,
  GetPublicationsQuery,
  GetPublicationsQueryVariables,
  Post,
} from "../../src/generated/graphql";
import { lensClient } from "../../src/helpers/lens/client";
import {
  getMediaImageSSR,
  useMediaImage,
} from "../../src/helpers/lens/hooks/useMediaImage";
import { useLensStore } from "../../src/helpers/lens/store";
import { AppId } from "../../src/helpers/lens/types";
import { PageMetadata } from "../../src/helpers/types";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const handle = query.handle;

  const profileQuery = await lensClient
    .query<GetProfileByHandleQuery, GetProfileByHandleQueryVariables>(
      GetProfileByHandleDocument,
      { handle }
    )
    .toPromise();

  const profile = profileQuery.data?.profiles.items[0];
  const title = profile?.name ? `${profile?.name} (@${handle})` : `@${handle}`;
  const metadata: PageMetadata = {
    title,
    description: profile?.bio ?? "",
    image: getMediaImageSSR(profile?.picture) ?? "",
  };

  if (!profile) return { props: { metadata } };

  const spacesQuery = await lensClient
    .query<GetPublicationsQuery, GetPublicationsQueryVariables>(
      GetPublicationsDocument,
      {
        profileId: profile.id,
        sources: [AppId.space],
      }
    )
    .toPromise();

  const spaces = spacesQuery.data?.publications.items;

  return {
    props: { metadata, profile, spaces },
  };
}

interface Props {
  metadata: PageMetadata;
  profile?: GetProfileByHandleQuery["profiles"]["items"][0];
  spaces?: Post[];
}

export default function User({ metadata, profile, spaces }: Props) {
  const router = useRouter();
  const handle = router.query.handle as string;

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

              {spaces && spaces.length > 0 && (
                <div className="w-full space-y-4 md:ml-12 pt-4">
                  <div className="flex items-center justify-center w-full space-x-4">
                    <div className="text-xl font-bold rounded-lg px-3 py-1">
                      Spaces
                    </div>
                  </div>

                  <div className="grid md:grid-cols-2 gap-2">
                    {spaces?.map((space) => (
                      <div key={space.id}>
                        <Link href={`/space/${space.id}`} passHref>
                          <a>
                            <SpaceCard space={space} />
                          </a>
                        </Link>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </>
  );
}

User.getLayout = getNavbarLayout;

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
