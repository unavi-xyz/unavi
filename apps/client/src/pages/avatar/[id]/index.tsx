import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";

import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import AvatarLayout from "../../../home/layouts/AvatarLayout/AvatarLayout";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";

export const getServerSideProps = async ({
  res,
  query,
}: GetServerSidePropsContext) => {
  res.setHeader(
    "Cache-Control",
    "public, s-maxage=60, stale-while-revalidate=600"
  );

  const props = await getPublicationProps(query.id as string);

  return {
    props,
  };
};

export default function Avatar(
  props: InferGetServerSidePropsType<typeof getServerSideProps>
) {
  return (
    <AvatarLayout {...props}>
      <div className="space-y-2">
        <div className="text-2xl font-bold">Description</div>
        <div className="text-lg text-outline">
          {props.publication?.metadata.description}
        </div>
      </div>
    </AvatarLayout>
  );
}

Avatar.getLayout = getNavbarLayout;
