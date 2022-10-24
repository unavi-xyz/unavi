FROM node:18 AS builder
WORKDIR /app

# Prune files
RUN yarn global add turbo
COPY . .
RUN turbo prune --scope=@wired-labs/host --docker

FROM node:18 AS installer
WORKDIR /app

# Install mediasoup dependencies
RUN \
	set -x \
	&& apt-get update \
	&& apt-get install -y net-tools build-essential python3 python3-pip valgrind

# Install NPM dependencies
COPY .gitignore .gitignore
COPY --from=builder /app/out/json/ .
COPY --from=builder /app/out/yarn.lock ./yarn.lock
RUN yarn install --frozen-lockfile

# Build the project and its dependencies
COPY --from=builder /app/out/full/ .
COPY turbo.json turbo.json
RUN yarn turbo run build --filter=@wired-labs/host...

FROM node:18 AS runner
WORKDIR /app

# Don't run production as root
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nodejs
USER nodejs
COPY --from=installer /app .

CMD node --experimental-specifier-resolution=node apps/host/build/server.js