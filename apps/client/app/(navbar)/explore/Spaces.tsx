import { fetchLatestSpaces } from "../../../src/server/helpers/fetchLatestSpaces";
import SpaceCard from "./SpaceCard";

export default async function Spaces() {
  const spaces = await fetchLatestSpaces(40);

  return spaces.map(({ id, metadata }) => (
    <SpaceCard key={id} id={id} metadata={metadata} sizes="512" />
  ));
}
