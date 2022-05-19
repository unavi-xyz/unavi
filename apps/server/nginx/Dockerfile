FROM nginx:alpine AS base
WORKDIR /app
ARG HOSTNAME
ENV HOSTNAME=${HOSTNAME}

FROM base AS generator
COPY nginx.conf .
RUN sed -i "s/example.com/${HOSTNAME}/g" nginx.conf

FROM base AS runner
COPY --from=generator /app/nginx.conf /etc/nginx/nginx.conf
CMD ["nginx", "-g", "daemon off;"]
