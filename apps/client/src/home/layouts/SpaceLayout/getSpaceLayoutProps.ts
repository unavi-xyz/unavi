import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";

export async function getSpaceLayoutProps(id: string) {
  const host =
    process.env.NODE_ENV === "development"
      ? "localhost:4000"
      : "host.thewired.space";

  const hostURL =
    process.env.NODE_ENV === "development"
      ? "http://localhost:4000"
      : "https://host.thewired.space";

  // Fetch publication data
  const publicationPromise = getPublicationProps(id as string);

  // Fetch player count
  try {
    const response = await fetch(`${hostURL}/playercount/${id}`);
    const playerCountText = await response.text();
    const playerCount = parseInt(playerCountText);

    const publicationProps = await publicationPromise;

    return {
      ...publicationProps,
      host,
      playerCount,
    };
  } catch (error) {
    console.error(error);

    const publicationProps = await publicationPromise;

    return {
      ...publicationProps,
      host,
      playerCount: null,
    };
  }
}
