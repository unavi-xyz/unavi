import {
  AttributeData,
  MetadataVersions,
  ProfileMetadata,
} from "@wired-labs/lens";
import { nanoid } from "nanoid";
import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";

import { useFetchData } from "../../../lib/ipfs/useFetchData";
import { useLens } from "../../../lib/lens/hooks/useLens";
import { useProfileByHandle } from "../../../lib/lens/hooks/useProfileByHandle";
import { useSetProfileMetadata } from "../../../lib/lens/hooks/useSetProfileMetadata";
import { PublicationProps } from "../../../lib/lens/utils/getPublicationProps";
import { trimHandle } from "../../../lib/lens/utils/trimHandle";
import Button from "../../../ui/base/Button";
import NavigationTab from "../../../ui/base/NavigationTab";
import Spinner from "../../../ui/base/Spinner";
import MetaTags from "../../../ui/MetaTags";
import AvatarCanvas from "./AvatarCanvas";

interface Props extends PublicationProps {
  children: React.ReactNode;
}

export default function AvatarLayout({
  children,
  metadata,
  publication,
}: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  const { handle } = useLens();
  const profile = useProfileByHandle(handle);
  const avatarUrl = useFetchData(publication?.metadata.content);
  const setProfileMetadata = useSetProfileMetadata(profile?.id);

  const [loading, setLoading] = useState(false);

  const author = trimHandle(publication?.profile.handle);
  const isAuthor = handle && handle === author;
  const attribute = profile?.attributes?.find((item) => item.key === "avatar");
  const isEquipped = attribute?.value === id;
  const disableEquipButton = !handle;

  async function handleEquipAvatar() {
    if (!profile) return;

    setLoading(true);

    const attributes =
      profile.attributes?.map((attribute) => {
        const data: AttributeData = {
          key: attribute.key,
          value: attribute.value,
          traitType: attribute.traitType ?? undefined,
          displayType: (attribute.displayType as any) ?? undefined,
        };
        return data;
      }) ?? [];

    const cover_picture: string =
      profile.coverPicture?.__typename === "MediaSet"
        ? profile.coverPicture.original.url
        : profile.coverPicture?.__typename === "NftImage"
        ? profile.coverPicture.uri
        : null;

    function addAttribute(key: string, value: any) {
      const newData = {
        key,
        value,
      };

      const currentIndex = attributes.findIndex(
        (attribute) => attribute.key === key
      );

      if (value === undefined) {
        if (currentIndex !== -1) {
          //remove attribute
          attributes.splice(currentIndex, 1);
        }
      } else if (currentIndex === -1) {
        //add attribute
        attributes.push(newData);
      } else {
        //update attribute
        attributes[currentIndex] = newData;
      }
    }

    addAttribute("avatar", id);

    const metadata: ProfileMetadata = {
      version: MetadataVersions.one,
      metadata_id: nanoid(),
      name: profile?.name ?? null,
      bio: profile?.bio ?? null,
      cover_picture,
      attributes,
    };

    try {
      await setProfileMetadata(metadata);

      router.push(`/user/${handle}`);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  async function handleUnequipAvatar() {
    if (!profile) return;

    setLoading(true);

    const attributes =
      (profile.attributes
        ?.map((attribute) => {
          if (attribute.key === "avatar") return null;

          const data: AttributeData = {
            key: attribute.key,
            value: attribute.value,
            traitType: attribute.traitType ?? undefined,
            displayType: (attribute.displayType as any) ?? undefined,
          };
          return data;
        })
        .filter((item) => item !== null) as AttributeData[]) ?? [];

    const cover_picture: string =
      profile.coverPicture?.__typename === "MediaSet"
        ? profile.coverPicture.original.url
        : profile.coverPicture?.__typename === "NftImage"
        ? profile.coverPicture.uri
        : null;

    const metadata: ProfileMetadata = {
      version: MetadataVersions.one,
      metadata_id: nanoid(),
      name: profile?.name ?? null,
      bio: profile?.bio ?? null,
      cover_picture,
      attributes,
    };

    try {
      await setProfileMetadata(metadata);

      router.push(`/user/${handle}`);
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <>
      <MetaTags
        title={metadata.title ?? id}
        description={metadata.description ?? undefined}
        image={metadata.image ?? undefined}
        card="summary_large_image"
      />

      <div className="mx-4 h-full">
        <div className="max-w-content mx-auto h-full w-full space-y-8 py-8">
          <div className="flex flex-col space-y-8 md:flex-row md:space-y-0 md:space-x-8">
            <div className="aspect-vertical bg-primaryContainer mx-auto w-full rounded-3xl md:mx-0 md:w-1/2">
              {avatarUrl ? (
                <AvatarCanvas url={avatarUrl} />
              ) : (
                <div className="bg-surfaceVariant flex h-full animate-pulse items-center justify-center rounded-3xl">
                  <Spinner />
                </div>
              )}
            </div>

            <div className="flex min-w-fit flex-col justify-between space-y-8 md:w-2/3">
              <div className="space-y-4">
                <div className="flex justify-center text-3xl font-black">
                  {publication?.metadata.name}
                </div>

                <div className="space-y-2">
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div>By</div>
                    <Link href={`/user/${author}`}>
                      <a className="cursor-pointer hover:underline">
                        @{author}
                      </a>
                    </Link>
                  </div>
                </div>
              </div>

              {isEquipped ? (
                <div className="group">
                  <Button
                    variant="outlined"
                    fullWidth
                    loading={loading}
                    disabled={disableEquipButton}
                    onClick={handleUnequipAvatar}
                  >
                    <div className="py-2 group-hover:hidden">
                      Avatar Equipped
                    </div>
                    <div className="hidden py-2 group-hover:block">Unequip</div>
                  </Button>
                </div>
              ) : (
                <Button
                  variant="filled"
                  fullWidth
                  loading={loading}
                  disabled={disableEquipButton}
                  onClick={handleEquipAvatar}
                >
                  <div className="py-2">Equip Avatar</div>
                </Button>
              )}
            </div>
          </div>

          <div className="space-y-4 pb-4">
            <div className="flex space-x-4">
              <NavigationTab href={`/avatar/${id}`} text="About" />

              {isAuthor && (
                <NavigationTab
                  href={`/avatar/${id}/settings`}
                  text="Settings"
                />
              )}
            </div>

            <div>{children}</div>
          </div>
        </div>
      </div>
    </>
  );
}
