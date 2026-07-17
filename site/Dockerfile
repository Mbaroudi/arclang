# Static site: everything (wasm compiler included) is prebuilt by
# tools/build_site.sh — the image just serves files.
FROM nginx:1.27-alpine

COPY nginx.conf /etc/nginx/conf.d/default.conf
RUN rm /usr/share/nginx/html/index.html
COPY . /usr/share/nginx/html/
RUN rm -f /usr/share/nginx/html/Dockerfile /usr/share/nginx/html/nginx.conf \
          /usr/share/nginx/html/fly.toml

EXPOSE 8080
