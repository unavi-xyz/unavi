import { GetServerSideProps, InferGetServerSidePropsType } from "next";

import { PublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import { getSpaceLayoutProps } from "../../../home/layouts/SpaceLayout/getSpaceLayoutProps";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";

export const getServerSideProps: GetServerSideProps<
  PublicationProps & { host: string; playerCount: number | null }
> = async ({ res, query }) => {
  res.setHeader(
    "Cache-Control",
    "public, s-maxage=60, stale-while-revalidate=604800"
  );

  const props = await getSpaceLayoutProps(query.id as string);

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
