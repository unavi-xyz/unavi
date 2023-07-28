import { eq } from "drizzle-orm";
import { Metadata } from "next";
import Image from "next/image";
import { notFound } from "next/navigation";

import { baseMetadata } from "@/app/metadata";
import { db } from "@/src/server/db/drizzle";
import { user } from "@/src/server/db/schema";
import { FixWith } from "@/src/server/db/types";
import { fetchLatestWorlds } from "@/src/server/helpers/fetchLatestWorlds";
import Avatar from "@/src/ui/Avatar";
import WorldCard from "@/src/ui/WorldCard";
import { isFromCDN } from "@/src/utils/isFromCDN";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import EditProfileButton from "./EditProfileButton";

type Params = { handle: string };

interface Props {
  params: Params;
}

async function queryUser(username: string) {
  const _foundUser = await db.query.user.findFirst({
    columns: { address: true, id: true },
    where: eq(user.username, username),
    with: { profile: true },
  });
  if (!_foundUser) return null;
  const foundUser: FixWith<typeof _foundUser, "profile"> = _foundUser;

  return foundUser;
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const username = params.handle.split("%40")[1]; // Remove the @ from the handle
  if (!username) return {};

  const foundUser = await queryUser(username);
  if (!foundUser) return {};

  const title = `@${username}`;
  const description = foundUser.profile?.bio ?? "";
  const image = foundUser.profile?.imageKey
    ? cdnURL(S3Path.profile(foundUser.id).image(foundUser.profile.imageKey))
    : undefined;

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

export default async function Handle({ params }: Props) {
  const username = params.handle.split("%40")[1]; // Remove the @ from the handle
  if (!username) return notFound();

  const foundUser = await queryUser(username);
  if (!foundUser) return notFound();

  const worlds = await fetchLatestWorlds(20, foundUser.id);

  const background = foundUser.profile?.backgroundKey
    ? cdnURL(
      S3Path.profile(foundUser.id).background(
        foundUser.profile.backgroundKey,
      ),
    )
    : undefined;

  const image = foundUser.profile?.imageKey
    ? cdnURL(S3Path.profile(foundUser.id).image(foundUser.profile.imageKey))
    : undefined;

  return (
    <>
      <div className="flex justify-center">
        <div className="max-w-content">
          <div className="aspect-[3/1] w-full bg-neutral-200 md:h-72 xl:rounded-2xl">
            <div className="relative h-full w-full object-cover">
              {background ? (
                isFromCDN(background) ? (
                  <Image
                    src={background}
                    priority
                    fill
                    sizes="100vw"
                    alt=""
                    className="h-full w-full object-cover xl:rounded-2xl"
                  />
                ) : (
                  <img
                    src={background}
                    sizes="100vw"
                    alt=""
                    className="h-full w-full object-cover xl:rounded-2xl"
                    crossOrigin="anonymous"
                  />
                )
              ) : null}
            </div>
          </div>

          <section className="flex justify-center px-4 md:px-0">
            <div className="flex w-full flex-col items-center space-y-2">
              <div className="relative z-10 -mt-16 flex w-32 rounded-full ring-4 ring-white">
                <Avatar src={image} circle uniqueKey={username} size={128} />

                <EditProfileButton
                  userId={foundUser.id}
                  username={username}
                  bio={foundUser.profile?.bio ?? undefined}
                  imageKey={foundUser.profile?.imageKey ?? undefined}
                  image={image}
                  backgroundKey={foundUser.profile?.backgroundKey ?? undefined}
                  background={background}
                />
              </div>

              <div className="flex w-full flex-col items-center space-y-2">
                <div className="text-2xl font-bold">@{username}</div>
                <div className="w-full overflow-x-hidden text-ellipsis text-center text-neutral-400">
                  {foundUser?.address}
                </div>
              </div>

              {foundUser.profile?.bio && (
                <div className="w-full whitespace-pre-line text-center">
                  {foundUser.profile.bio}
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
