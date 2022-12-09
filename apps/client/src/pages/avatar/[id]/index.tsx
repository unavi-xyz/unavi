import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";

import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import AvatarLayout from "../../../home/layouts/AvatarLayout/AvatarLayout";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import { getAvatarStats } from "../../../server/helpers/getAvatarStats";

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

  const id = query.id as string;

  const publicationPropsPromise = getPublicationProps(id);
  const avatarStatsPromise = getAvatarStats(id);

  const publicationProps = await publicationPropsPromise;
  const avatarStats = await avatarStatsPromise;

  return {
    props: {
      ...publicationProps,
      avatarStats,
    },
  };
};

export default function Avatar(
  props: InferGetServerSidePropsType<typeof getServerSideProps>
) {
  return (
    <AvatarLayout {...props}>
      <div className="space-y-4">
        {props.publication?.metadata.description && (
          <div className="space-y-2">
            <div className="text-2xl font-bold">Description</div>
            <div className="whitespace-pre-line text-lg text-neutral-500">
              {props.publication.metadata.description}
            </div>
          </div>
        )}

        <div className="space-y-2">
          <div className="text-2xl font-bold">Stats</div>

          <div className="flex text-lg">
            <div>
              {["Polygons", "Materials", "Meshes", "Skins", "Bones"].map(
                (title) => (
                  <div key={title} className="font-bold">
                    {title}
                  </div>
                )
              )}
            </div>

            <div className="pl-8">
              {[
                props.avatarStats.polygonCount,
                props.avatarStats.materialCount,
                props.avatarStats.meshCount,
                props.avatarStats.skinCount,
                props.avatarStats.boneCount,
              ].map((stat, i) => (
                <div key={i} className="text-neutral-500">
                  {stat}
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </AvatarLayout>
  );
}

Avatar.getLayout = getNavbarLayout;
