FROM --platform=linux/amd64 stoplight/prism:latest

COPY ./openapi.yaml /tmp/openapi.yaml

CMD ["mock", "-d", "-h", "0.0.0.0", "-p", "4010", "/tmp/openapi.yaml"]