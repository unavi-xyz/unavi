interface Props {
  description: string;
}

export default async function About({ description }: Props) {
  return <div className="whitespace-pre-line">{description}</div>;
}
