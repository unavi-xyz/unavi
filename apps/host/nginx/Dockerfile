FROM nginx:alpine
ARG HOSTNAME
ENV HOSTNAME=${HOSTNAME}
COPY nginx.conf /etc/nginx/
RUN sed -i "s/example.com/${HOSTNAME}/g" /etc/nginx/nginx.conf
CMD ["nginx", "-g", "daemon off;"]
