import { Post, useGetPublicationQuery } from "@wired-labs/lens";

import SpaceCard from "./SpaceCard";

interface Props {
  spaceId: string;
  sizes?: string;
  animateEnter?: boolean;
}

export default function SpaceIdCard({
  spaceId,
  sizes,
  animateEnter = false,
}: Props) {
  const [{ data }] = useGetPublicationQuery({
    variables: {
      request: {
        publicationId: spaceId,
      },
    },
  });

  if (!data) return null;

  return (
    <SpaceCard
      space={data.publication as Post}
      sizes={sizes}
      animateEnter={animateEnter}
    />
  );
}
