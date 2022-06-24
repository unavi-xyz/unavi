import {
  GetPublicationDocument,
  GetPublicationQuery,
  GetPublicationQueryVariables,
  Publication,
} from "../../generated/graphql";
import { PageMetadata } from "../../types";
import { lensClient } from "./client";
import { getMediaImageSSR } from "./hooks/useMediaImage";
import { AppId } from "./types";

export interface PublicationProps {
  metadata: PageMetadata;
  publication: Publication | undefined;
}

export async function getPublicationProps(id: string) {
  const { data } = await lensClient
    .query<GetPublicationQuery, GetPublicationQueryVariables>(
      GetPublicationDocument,
      { publicationId: id }
    )
    .toPromise();

  const publication = data?.publication as Publication | undefined;

  const title = `${publication?.metadata.name ?? id}`;

  const publicationType =
    publication?.appId === AppId.space
      ? "Space"
      : publication?.appId === AppId.avatar
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
