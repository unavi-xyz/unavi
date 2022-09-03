import { AppId, getMediaImageUri } from "@wired-labs/lens";
import {
  GetPublicationDocument,
  GetPublicationQuery,
  GetPublicationQueryVariables,
  Publication,
} from "@wired-labs/lens";

import { PageMetadata } from "../../types";
import { parseUri } from "../../utils/parseUri";
import { lensClient } from "./client";

export interface PublicationProps {
  metadata: PageMetadata;
  publication: Publication | null;
  image: string | null;
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

  const publication = (data?.publication as Publication | undefined) ?? null;

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

  const image = publication
    ? parseUri(getMediaImageUri(publication.metadata.media[0]))
    : null;

  const metadata: PageMetadata = {
    title,
    description,
    image,
  };

  const props: PublicationProps = { metadata, publication, image };
  return props;
}
