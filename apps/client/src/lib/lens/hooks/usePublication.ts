import { useGetPublicationQuery } from "../../../generated/graphql";

export function usePublication(publicationId: string | undefined) {
  const [{ data }] = useGetPublicationQuery({
    variables: { publicationId },
    pause: !publicationId,
  });

  return data?.publication;
}
