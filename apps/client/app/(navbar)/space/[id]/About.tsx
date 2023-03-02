import { fetchSpace } from "../../../../src/server/helpers/fetchSpace";

export const revalidate = 60;

type Params = { id: string };

export default async function About({ params }: { params: Params }) {
  const { id } = params;
  const spaceId = parseInt(id);
  const space = await fetchSpace(spaceId);

  if (!space) return null;

  return <div className="whitespace-pre-line">{space.metadata?.description}</div>;
}
