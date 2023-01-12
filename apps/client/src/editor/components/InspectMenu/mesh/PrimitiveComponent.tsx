interface Props {
  primitiveId: string;
}

export default function PrimitiveComponent({ primitiveId }: Props) {
  return <div>Primitive: {primitiveId}</div>;
}
