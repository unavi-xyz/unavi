import {
  getPublicationProps,
  PublicationProps,
} from "../../../lib/lens/utils/getPublicationProps";

export interface SpaceLayoutProps extends PublicationProps {
  playerCount?: number;
  host?: string;
  children: React.ReactNode;
}

export async function getSpaceLayoutProps(id: string) {
  // Fetch publication data
  const publicationProps = await getPublicationProps(id as string);

  return publicationProps;

  // // Fetch player count
  // ! Causes issues if cant reach server, ping player count from client
  // const host =
  //   process.env.NODE_ENV === "production"
  //     ? publicationProps.publication?.profile.attributes?.find(
  //         (item) => item.key === "host"
  //       )?.value ?? DEFAULT_HOST
  //     : "localhost:4000";

  // try {
  //   const playerCountRes = await fetch(
  //     `${HTTP}://${host}/space/${id}/player_count`
  //   );
  //   const playerCount = await playerCountRes.json();

  //   return {
  //     ...publicationProps,
  //     host,
  //     playerCount,
  //   };
  // } catch {
  //   return {
  //     ...publicationProps,
  //     host,
  //     playerCount: null,
  //   };
  // }
}
