import { AppId, getMediaImageUri } from "@wired-xr/lens";
import {
  GetPublicationDocument,
  GetPublicationQuery,
  GetPublicationQueryVariables,
  Publication,
} from "@wired-xr/lens";

import { PageMetadata } from "../../types";
import { parseUri } from "../../utils/parseUri";
import { lensClient } from "./client";

export interface PublicationProps {
  metadata: PageMetadata;
  publication: Publication | undefined;
  image: string | undefined;
}

export async function getPublicationProps(
  publicationId: string
): Promise<PublicationProps> {
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
      : undefined;

  const description =
    publication?.metadata.description ?? publication?.profile.handle
      ? `${publicationType} by @${publication?.profile.handle}`
      : undefined;

  const image = publication
    ? parseUri(getMediaImageUri(publication.metadata.media[0]))
    : undefined;

  const metadata: PageMetadata = {
    title,
    description,
    image,
  };

  const props: PublicationProps = { metadata, publication, image };
  return props;
}
