import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";

import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";

export const getServerSideProps = async ({
  res,
  query,
}: GetServerSidePropsContext) => {
  res.setHeader(
    "Cache-Control",
    "public, s-maxage=60, stale-while-revalidate=604800"
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
        <div className="whitespace-pre-line text-lg text-outline">
          {props.publication?.metadata.description}
        </div>
      </div>
    </SpaceLayout>
  );
}

Space.getLayout = getNavbarLayout;
