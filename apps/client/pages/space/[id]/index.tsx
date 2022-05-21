import { NextPageContext } from "next";

import { getSpaceLayout } from "../../../src/components/layouts/SpaceLayout/SpaceLayout";
import { usePublication } from "../../../src/helpers/lens/hooks/usePublication";

interface Props {
  id: string;
}

export async function getServerSideProps(ctx: NextPageContext) {
  const id = ctx.query.id as string;
  const props: Props = { id };

  return {
    props,
  };
}

export default function Space({ id }: Props) {
  const publication = usePublication(id);

  return (
    <div className="space-y-2">
      <div className="text-2xl font-bold">Description</div>
      <div className="text-lg text-outline">
        {publication?.metadata.description}
      </div>
    </div>
  );
}

Space.getLayout = getSpaceLayout;
