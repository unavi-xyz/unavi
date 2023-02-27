import { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Suspense } from "react";

import Avatar from "../../../../src/home/Avatar";
import { fetchProfile } from "../../../../src/server/helpers/fetchProfile";
import { fetchProfileFromAddress } from "../../../../src/server/helpers/fetchProfileFromAddress";
import { getServerSession } from "../../../../src/server/helpers/getServerSession";
import Card from "../../../../src/ui/Card";
import { hexDisplayToNumber, numberToHexDisplay } from "../../../../src/utils/numberToHexDisplay";
import Spaces from "./Spaces";

type Params = { id: string };

export async function generateMetadata({ params: { id } }: { params: Params }): Promise<Metadata> {
  const isAddress = id.length === 42;

  const profile = isAddress
    ? await fetchProfileFromAddress(id)
    : await fetchProfile(hexDisplayToNumber(id));

  if (isAddress && !profile) {
    return {
      title: id.substring(0, 6) + "...",
      description: "",
      openGraph: {
        type: "profile",
        username: id,
        description: "",
      },
    };
  }

  if (!profile) return {};

  return {
    title: profile.handle?.string ?? `User ${numberToHexDisplay(profile.id)}`,
    description: profile.metadata?.description ?? "",
    openGraph: {
      type: "profile",
      username: profile.handle?.full,
      firstName: profile.handle?.string,
      description: profile.metadata?.description ?? "",
    },
  };
}

export default async function User({ params: { id } }: { params: Params }) {
  const session = await getServerSession();

  const isAddress = id.length === 42;

  const profile = isAddress
    ? await fetchProfileFromAddress(id)
    : await fetchProfile(hexDisplayToNumber(id));

  if (!isAddress && !profile) notFound();

  const isUser = isAddress ? session?.address === id : session?.address === profile?.owner;

  return (
    <div className="max-w-content mx-auto">
      <div className="h-48 w-full bg-neutral-200 md:h-64 xl:rounded-xl">
        <div className="relative h-full w-full object-cover">
          {profile?.metadata?.animation_url && (
            <img
              src={profile.metadata.animation_url}
              alt=""
              className="h-full w-full object-cover xl:rounded-xl"
              crossOrigin="anonymous"
            />
          )}
        </div>
      </div>

      <section className="flex justify-center px-4 pb-6 md:px-0">
        <div className="flex w-full flex-col items-center space-y-2">
          <div className="z-10 -mt-16 flex w-32 rounded-full ring-4 ring-white">
            <Avatar
              src={profile?.metadata?.image}
              circle
              uniqueKey={profile?.handle?.full ?? id}
              size={128}
            />
          </div>

          <div className="flex w-full flex-col items-center">
            {profile?.handle ? (
              <div>
                <span className="text-2xl font-black">{profile.handle.string}</span>
                <span className="text-xl font-bold text-neutral-400">
                  #{profile.handle.id.toString().padStart(4, "0")}
                </span>
              </div>
            ) : null}

            <div className="w-full overflow-x-hidden text-ellipsis text-center text-neutral-400">
              {isAddress ? id : profile?.owner}
            </div>
          </div>

          {profile?.metadata?.description && (
            <div className="w-full whitespace-pre-line text-center">
              {profile.metadata.description}
            </div>
          )}

          {isUser && (
            <div className="flex w-full justify-center space-x-2">
              <Link
                href="/settings"
                className="rounded-md px-10 py-1.5 font-bold ring-1 ring-neutral-700 transition hover:bg-neutral-200 active:bg-neutral-300"
              >
                Edit profile
              </Link>
            </div>
          )}
        </div>
      </section>

      <div className="grid grid-cols-1 gap-4 px-4 sm:grid-cols-2 md:grid-cols-3">
        <Suspense
          fallback={Array.from({ length: 3 }).map((_, i) => (
            <Card key={i} loading />
          ))}
        >
          {/* @ts-expect-error Server Component */}
          <Spaces owner={isAddress ? id : profile.owner} />
        </Suspense>
      </div>
    </div>
  );
}
