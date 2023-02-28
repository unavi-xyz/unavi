import { notFound } from "next/navigation";

import { fetchSpace } from "../../../../src/server/helpers/fetchSpace";

type Params = { id: string };

export default async function About({ params: { id } }: { params: Params }) {
  const spaceId = parseInt(id);
  const space = await fetchSpace(spaceId);

  if (!space) notFound();

  return <div className="whitespace-pre-line">{space.metadata?.description}</div>;
}
