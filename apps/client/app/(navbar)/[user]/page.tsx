import { Metadata } from "next";
import Image from "next/image";
import { notFound } from "next/navigation";

import { baseMetadata } from "@/app/metadata";
import {
  FetchedWorld,
  fetchLatestWorlds,
} from "@/src/server/helpers/fetchLatestWorlds";
import { fetchProfile } from "@/src/server/helpers/fetchProfile";
import Avatar from "@/src/ui/Avatar";
import WorldCard from "@/src/ui/WorldCard";
import { isFromCDN } from "@/src/utils/isFromCDN";
import { parseIdentity } from "@/src/utils/parseIdentity";

import EditProfileButton from "./EditProfileButton";

type Params = { user: string };

interface Props {
  params: Params;
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const identity = parseIdentity(params.user);
  const profile = await fetchProfile(identity);
  if (!profile) return {};

  const description = profile.bio ?? "";
  const image = profile.image;

  let title: string | undefined = undefined;
  let username: string | undefined = undefined;

  if (profile.type === "db") {
    title = profile.username;
    username = profile.username;
  } else if (profile.type === "did") {
    title = profile.did;
  }

  return {
    description,
    openGraph: {
      ...baseMetadata.openGraph,
      description,
      images: image ? [{ url: image }] : undefined,
      title,
      type: "profile",
      username,
    },
    title,
    twitter: {
      ...baseMetadata.twitter,
      card: image ? "summary_large_image" : "summary",
      description,
      images: image ? [image] : undefined,
      title,
    },
  };
}

export default async function User({ params }: Props) {
  const identity = parseIdentity(params.user);
  const profile = await fetchProfile(identity);
  if (!profile) notFound();

  const image = profile.image;
  const background = profile.background;

  let id: string | undefined = undefined;
  let username: string | undefined = undefined;
  let imageKey: string | undefined = undefined;
  let backgroundKey: string | undefined = undefined;
  let worlds: FetchedWorld[] = [];

  if (profile.type === "db") {
    id = profile.id;
    username = profile.username;
    imageKey = profile.imageKey;
    backgroundKey = profile.backgroundKey;

    worlds = await fetchLatestWorlds(20, id);
  }

  return (
    <>
      <div className="flex justify-center">
        <div className="max-w-content">
          <div className="aspect-[3/1] w-full bg-neutral-200 lg:rounded-2xl">
            <div className="relative h-full w-full object-cover">
              {background ? (
                isFromCDN(background) ? (
                  <Image
                    src={background}
                    priority
                    fill
                    sizes="100vw"
                    alt=""
                    className="h-full w-full object-cover lg:rounded-2xl"
                  />
                ) : (
                  <img
                    src={background}
                    sizes="100vw"
                    alt=""
                    className="h-full w-full object-cover lg:rounded-2xl"
                    crossOrigin="anonymous"
                  />
                )
              ) : null}
            </div>
          </div>

          <section className="flex justify-center px-4 md:px-0">
            <div className="flex w-full flex-col items-center space-y-2">
              <div className="relative z-10 -mt-16 flex w-32 rounded-full ring-4 ring-white">
                <Avatar src={image} circle uniqueKey={profile.did} size={128} />

                {id && username && (
                  <EditProfileButton
                    userId={id}
                    username={username}
                    bio={profile?.bio}
                    imageKey={imageKey}
                    image={image}
                    did={profile.did}
                    backgroundKey={backgroundKey}
                    background={background}
                  />
                )}
              </div>

              <div className="flex w-full flex-col items-center space-y-2">
                <div className="text-2xl font-bold">@{username}</div>
                <div className="w-full overflow-x-hidden text-ellipsis text-center text-neutral-400">
                  {profile?.address}
                </div>
              </div>

              {profile?.bio && (
                <div className="w-full whitespace-pre-line text-center">
                  {profile.bio}
                </div>
              )}
            </div>
          </section>
        </div>
      </div>

      <div className="flex justify-center pb-8 pt-4">
        <div className="max-w-content mx-4 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {worlds.map(({ id, uri, metadata }) => (
            <WorldCard
              key={id}
              id={id}
              uri={uri}
              metadata={metadata}
              sizes="(min-width: 1320px) 33vw, (min-width: 768px) 50vw, 100vw"
            />
          ))}
        </div>
      </div>
    </>
  );
}
