import { nanoid } from "nanoid";

import { AttributeData, MetadataVersions, ProfileMetadata } from "@wired-xr/lens";

import { Profile } from "../../generated/graphql";

export function createProfileMetadata(profile: Profile) {
  const cover_picture =
    profile.coverPicture?.__typename === "MediaSet"
      ? profile.coverPicture.original.url
      : profile.coverPicture?.__typename === "NftImage"
      ? profile.coverPicture.uri
      : null;

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

  const metadata: ProfileMetadata = {
    version: MetadataVersions.one,
    metadata_id: nanoid(),
    name: profile.name ?? null,
    bio: profile.bio ?? null,
    cover_picture,
    attributes,
  };

  function updateAttribute(key: string, value: any) {
    const newData = {
      key,
      value,
    };

    const currentIndex = metadata.attributes.findIndex((attribute) => attribute.key === key);

    if (value === undefined) {
      if (currentIndex !== -1) {
        //remove attribute
        metadata.attributes.splice(currentIndex, 1);
      }
    } else if (currentIndex === -1) {
      //add attribute
      metadata.attributes.push(newData);
    } else {
      //update attribute
      metadata.attributes[currentIndex] = newData;
    }
  }

  return { metadata, updateAttribute };
}
