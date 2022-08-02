FROM node:16 as base
ARG SCOPE
ARG DB_USER
ARG DB_PASSWORD
ARG DB_NAME
ARG DB_HOST
ARG DB_PORT

ENV SCOPE=${SCOPE}
ENV DB_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
ENV YARN_CACHE_FOLDER=.yarn-cache

FROM base AS pruner
# turborepo combines everything into a single lock file
# we need to run turbo prune to remove all deps we dont need
RUN yarn global add turbo
COPY . .
RUN turbo prune --scope=${SCOPE} --docker

FROM base AS dev-deps
COPY --from=pruner /out/json/ .
COPY --from=pruner /out/yarn.lock ./yarn.lock
RUN yarn install --frozen-lockfile

FROM base AS prod-deps
COPY --from=pruner /out/json/ .
COPY --from=pruner /out/yarn.lock ./yarn.lock
COPY --from=dev-deps /${YARN_CACHE_FOLDER} /${YARN_CACHE_FOLDER}
RUN yarn install --frozen-lockfile --production --prefer-offline --ignore-scripts
RUN rm -rf /${YARN_CACHE_FOLDER}

FROM base AS builder
COPY --from=dev-deps . .
COPY --from=pruner /out/full/ .

RUN yarn turbo run codegen --filter=${SCOPE}...
RUN yarn turbo run build --filter=${SCOPE}...
RUN find . -name node_modules | xargs rm -rf

FROM base AS runner
COPY --from=dev-deps . .
COPY --from=prod-deps . .
COPY --from=builder . .
