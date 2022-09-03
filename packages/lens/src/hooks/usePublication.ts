import { useGetPublicationQuery } from "../..";

export function usePublication(publicationId: string | undefined) {
  const [{ data }] = useGetPublicationQuery({
    variables: { request: { publicationId } },
    pause: !publicationId,
  });

  return data?.publication;
}
