import { AppId, getMediaImageSSR } from "@wired-xr/lens";
import {
  GetPublicationDocument,
  GetPublicationQuery,
  GetPublicationQueryVariables,
  Publication,
} from "@wired-xr/lens";

import { PageMetadata } from "../../types";
import { lensClient } from "./client";

export interface PublicationProps {
  metadata: PageMetadata;
  publication: Publication | undefined;
}

export async function getPublicationProps(publicationId: string) {
  const { data } = await lensClient
    .query<GetPublicationQuery, GetPublicationQueryVariables>(
      GetPublicationDocument,
      {
        request: { publicationId },
      }
    )
    .toPromise();

  const publication = data?.publication as Publication | undefined;

  const title = `${publication?.metadata.name ?? publicationId}`;

  const publicationType =
    publication?.appId === AppId.Space
      ? "Space"
      : publication?.appId === AppId.Avatar
      ? "Avatar"
      : "";

  const description =
    publication?.metadata.description ?? publication?.profile.handle
      ? `${publicationType} by @${publication?.profile.handle}`
      : "";

  const image = getMediaImageSSR(publication?.metadata.media[0]) ?? "";

  const metadata: PageMetadata = {
    title,
    description,
    image,
  };

  const props: PublicationProps = { metadata, publication };

  return props;
}
