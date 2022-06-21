import {
  PublicationProps,
  getPublicationProps,
} from "../../../helpers/lens/getPublicationProps";
import { DEFAULT_HOST } from "../../app/SpaceProvider";

const HTTP = process.env.NODE_ENV === "production" ? "https" : "http";

export interface SpaceLayoutProps extends PublicationProps {
  playerCount: number;
  children: React.ReactNode;
}

export async function getSpaceLayoutProps(id: string) {
  const publicationProps = await getPublicationProps(id as string);

  const host =
    process.env.NODE_ENV === "production"
      ? publicationProps.publication?.profile.attributes?.find(
          (item) => item.key === "host"
        )?.value ?? DEFAULT_HOST
      : "localhost:4000";

  const playerCountRes = await fetch(
    `${HTTP}://${host}/space/${id}/player_count`
  );
  const playerCount = Number(await playerCountRes.text());

  return {
    ...publicationProps,
    playerCount,
  };
}
