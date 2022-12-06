import { AttributeData, ProfileMetadata, ProfileMetadataVersions } from "lens";
import { nanoid } from "nanoid";
import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";

import { useLens } from "../../../client/lens/hooks/useLens";
import { useProfileByHandle } from "../../../client/lens/hooks/useProfileByHandle";
import { useSetProfileMetadata } from "../../../client/lens/hooks/useSetProfileMetadata";
import { PublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { trimHandle } from "../../../client/lens/utils/trimHandle";
import Button from "../../../ui/Button";
import NavigationTab from "../../../ui/NavigationTab";
import { isFromCDN } from "../../../utils/isFromCDN";
import MetaTags from "../../MetaTags";

interface Props extends PublicationProps {
  children: React.ReactNode;
}

export default function AvatarLayout({
  children,
  metadata,
  image,
  publication,
}: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  const { handle } = useLens();
  const profile = useProfileByHandle(handle);
  const setProfileMetadata = useSetProfileMetadata(profile?.id);

  const [loading, setLoading] = useState(false);
  const [changedEquipped, setChangedEquipped] = useState<boolean | null>(null);

  const author = trimHandle(publication?.profile.handle);
  const isAuthor = handle && handle === author;
  const attribute = profile?.attributes?.find((item) => item.key === "avatar");
  const isEquipped =
    changedEquipped !== null ? changedEquipped : attribute?.value === id;
  const disableEquipButton = !handle;

  async function handleEquipAvatar() {
    if (!profile || loading) return;

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
      version: ProfileMetadataVersions.one,
      metadata_id: nanoid(),
      name: profile?.name ?? null,
      bio: profile?.bio ?? null,
      cover_picture,
      attributes,
    };

    try {
      await setProfileMetadata(metadata);
      setChangedEquipped(true);
      router.reload();
    } catch (err) {
      console.error(err);
    }

    setLoading(false);
  }

  async function handleUnequipAvatar() {
    if (!profile || loading) return;

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
      version: ProfileMetadataVersions.one,
      metadata_id: nanoid(),
      name: profile?.name ?? null,
      bio: profile?.bio ?? null,
      cover_picture,
      attributes,
    };

    try {
      await setProfileMetadata(metadata);
      setChangedEquipped(false);
      router.reload();
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
          <div className="flex flex-col items-center space-y-8 md:flex-row md:items-stretch md:space-y-0 md:space-x-8">
            <div className="aspect-vertical h-full w-1/2 rounded-3xl bg-primaryContainer">
              <div className="relative h-full w-full object-cover">
                {image &&
                  (isFromCDN(image) ? (
                    <Image
                      src={image}
                      priority
                      fill
                      sizes="425px"
                      alt=""
                      className="rounded-3xl object-cover"
                    />
                  ) : (
                    <img
                      src={image}
                      alt=""
                      className="h-full w-full rounded-3xl object-cover"
                      crossOrigin="anonymous"
                    />
                  ))}
              </div>
            </div>

            <div className="flex min-w-fit flex-col justify-between space-y-8 md:w-2/3">
              <div className="space-y-4">
                <div className="flex justify-center text-3xl font-black">
                  {publication?.metadata.name}
                </div>

                <div className="flex justify-center space-x-1 font-bold">
                  <div className="text-outline">By</div>
                  <Link href={`/user/${author}`}>
                    <div className="cursor-pointer hover:underline">
                      @{author}
                    </div>
                  </Link>
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
