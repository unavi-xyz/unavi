import {
  GetPublicationDocument,
  GetPublicationQuery,
  GetPublicationQueryVariables,
  Publication,
} from "@wired-labs/lens";

import { PageMetadata } from "../../../home/MetaTags";
import { lensClient } from "../../../server/lens";
import { getMediaURL } from "../../../utils/getMediaURL";

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
      { request: { publicationId } }
    )
    .toPromise();

  const publication = (data?.publication as Publication | undefined) ?? null;
  const title = publication?.metadata.name ?? `Space ${publicationId}`;
  const description = publication?.metadata.description ?? null;
  const image = getMediaURL(publication?.metadata.media[0]);

  const metadata: PageMetadata = {
    title,
    description,
    image,
  };

  const props: PublicationProps = { metadata, publication, image };
  return props;
}
