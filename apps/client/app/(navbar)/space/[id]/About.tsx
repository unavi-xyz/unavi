import { fetchSpace } from "@/src/server/helpers/fetchSpace";

interface Props {
  id: number;
}

export default async function About({ id }: Props) {
  const space = await fetchSpace(id);

  if (!space) return null;

  return <div className="whitespace-pre-line">{space.metadata?.description}</div>;
}
