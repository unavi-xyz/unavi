import { DEFAULT_HOST } from "../../../../pages/app/[id]";
import { PublicationProps, getPublicationProps } from "../../../lib/lens/getPublicationProps";

const HTTP = process.env.NODE_ENV === "production" ? "https" : "http";

export interface SpaceLayoutProps extends PublicationProps {
  playerCount?: number;
  host?: string;
  children: React.ReactNode;
}

export async function getSpaceLayoutProps(id: string) {
  const publicationProps = await getPublicationProps(id as string);

  const host =
    process.env.NODE_ENV === "production"
      ? publicationProps.publication?.profile.attributes?.find((item) => item.key === "host")
          ?.value ?? DEFAULT_HOST
      : "localhost:4000";

  try {
    const playerCountRes = await fetch(`${HTTP}://${host}/space/${id}/player_count`);
    const playerCount = await playerCountRes.json();

    return {
      ...publicationProps,
      host,
      playerCount,
    };
  } catch {
    return {
      ...publicationProps,
      host,
      playerCount: null,
    };
  }
}
