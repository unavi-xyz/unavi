import {
  GetPublicationDocument,
  GetPublicationQuery,
  GetPublicationQueryVariables,
} from "../../../generated/graphql";
import { lensClient } from "../../../helpers/lens/client";
import { getMediaImageSSR } from "../../../helpers/lens/hooks/useMediaImage";
import { PageMetadata } from "../../../helpers/types";

export interface SpaceLayoutProps {
  metadata: PageMetadata;
  publication: GetPublicationQuery["publication"] | undefined;
}

export async function getSpaceLayoutProps(id: string) {
  const { data } = await lensClient
    .query<GetPublicationQuery, GetPublicationQueryVariables>(
      GetPublicationDocument,
      { publicationId: id }
    )
    .toPromise();

  const publication = data?.publication;
  const metadata: PageMetadata = {
    title: publication?.metadata.name ?? "",
    description: publication?.metadata.description ?? "",
    image: getMediaImageSSR(publication?.metadata.media[0]) ?? "",
  };

  const props: SpaceLayoutProps = { metadata, publication: data?.publication };

  return props;
}
