import { Post, useGetPublicationQuery } from "lens";

import { HIDDEN_MESSAGE } from "../../client/lens/constants";
import SpaceCard from "./SpaceCard";

interface Props {
  spaceId: string;
  sizes?: string;
  animateEnter?: boolean;
}

export default function SpaceIdCard({ spaceId, sizes, animateEnter = false }: Props) {
  const [{ data }] = useGetPublicationQuery({
    variables: { request: { publicationId: spaceId } },
  });

  if (!data || data.publication?.metadata.content === HIDDEN_MESSAGE) return null;

  return <SpaceCard space={data.publication as Post} sizes={sizes} animateEnter={animateEnter} />;
}
