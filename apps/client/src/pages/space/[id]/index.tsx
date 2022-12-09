import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";

import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";

export const getServerSideProps = async ({
  res,
  query,
}: GetServerSidePropsContext) => {
  const ONE_HOUR_IN_SECONDS = 60 * 60;
  const ONE_WEEK_IN_SECONDS = ONE_HOUR_IN_SECONDS * 24 * 7;

  res.setHeader(
    "Cache-Control",
    `public, s-maxage=${ONE_HOUR_IN_SECONDS}, stale-while-revalidate=${ONE_WEEK_IN_SECONDS}`
  );

  const props = await getPublicationProps(query.id as string);

  return {
    props,
  };
};

export default function Space(
  props: InferGetServerSidePropsType<typeof getServerSideProps>
) {
  return (
    <SpaceLayout {...props}>
      <div className="space-y-2">
        <div className="text-2xl font-bold">Description</div>
        <div className="whitespace-pre-line text-lg text-neutral-500">
          {props.publication?.metadata.description}
        </div>
      </div>
    </SpaceLayout>
  );
}

Space.getLayout = getNavbarLayout;
