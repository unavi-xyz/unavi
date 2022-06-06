import { nanoid } from "nanoid";
import Link from "next/link";
import { useRouter } from "next/router";
import { useState } from "react";

import { PublicationProps } from "../../../helpers/lens/getPublicationProps";
import { useProfileByHandle } from "../../../helpers/lens/hooks/useProfileByHandle";
import { useSetProfileMetadata } from "../../../helpers/lens/hooks/useSetProfileMetadata";
import { useLensStore } from "../../../helpers/lens/store";
import {
  AttributeData,
  MetadataVersions,
  ProfileMetadata,
} from "../../../helpers/lens/types";
import Button from "../../base/Button";
import NavigationTab from "../../base/NavigationTab";
import MetaTags from "../../ui/MetaTags";
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

  const handle = useLensStore((state) => state.handle);
  const profile = useProfileByHandle(handle);
  const setProfileMetadata = useSetProfileMetadata(profile?.id);

  const [loading, setLoading] = useState(false);

  const isAuthor = handle && handle === publication?.profile.handle;
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
        title={metadata.title}
        description={metadata.description}
        image={metadata.image}
        imageWidth="357.11px"
        imageHeight="595.2px"
        card="summary_large_image"
      />

      <div className="mx-4 h-full">
        <div className="max-w mx-auto py-8 w-full h-full space-y-8">
          <div className="flex flex-col md:flex-row space-y-8 md:space-y-0 md:space-x-8">
            <div className="w-1/2 rounded-3xl aspect-vertical bg-primaryContainer">
              {publication && (
                <AvatarCanvas uri={publication.metadata.content} />
              )}
            </div>

            <div className="md:w-2/3 min-w-fit flex flex-col justify-between space-y-8">
              <div className="space-y-4">
                <div className="font-black text-3xl flex justify-center">
                  {publication?.metadata.name}
                </div>

                <div className="space-y-2">
                  <div className="font-bold flex space-x-1 justify-center md:justify-start">
                    <div>By</div>
                    <Link href={`/user/${publication?.profile.handle}`}>
                      <a className="hover:underline cursor-pointer">
                        @{publication?.profile.handle}
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
                    <div className="py-2 hidden group-hover:block">Unequip</div>
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
