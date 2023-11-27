FROM ubuntu:jammy

RUN apt-get update --fix-missing || \
    (sleep 5 && apt-get update --fix-missing) || \
    (sleep 10 && apt-get update --fix-missing) && \
    apt-get install -y curl ca-certificates || \
    (sleep 5 && apt-get install -y curl ca-certificates) || \
    (sleep 10 && apt-get install -y curl ca-certificates) && \
    rm -rf /var/lib/apt/lists/* 

RUN mkdir -p /da/ && \
    groupadd -r avail && \
    useradd --no-log-init -r -g avail avail && \
    chown -R avail:avail /da

COPY entrypoint.sh /da/entrypoint.sh
RUN chmod +x /da/entrypoint.sh

USER avail:avail
WORKDIR /da

ARG NODE_CLIENT_WS
ARG NODE_CLIENT_RPC
ARG APP_ID
ARG LIBP2P_PORT

RUN echo "APP_ID is set to: $APP_ID, Node url: $NODE_CLIENT_WS"

RUN curl -L https://raw.githubusercontent.com/availproject/availproject.github.io/c804aa520b66838209bb1bafbf7ffefdb249a2ac/static/kate/avail-light-1.4.3/config.yaml --output config.yaml && \
    echo "app_id = ${APP_ID}" >> config.yaml && \
    sed -i "s#full_node_ws = .*#full_node_ws = ['$NODE_CLIENT_WS']#" config.yaml && \
    sed -i "s#full_node_rpc= .*#full_node_rpc = ['$NODE_CLIENT_RPC']#" config.yaml && \
    sed -i "s#http_server_port = .*#http_server_port = 7000#" config.yaml && \
    sed -i "s#libp2p_port = .*#libp2p_port = ${LIBP2P_PORT}#" config.yaml && \
    sed -i "s#http_server_host = .*#http_server_host = '0.0.0.0'#" config.yaml && \
    sed -i "s#bootstraps = .*#bootstraps = [['12D3KooWBkLsNGaD3SpMaRWtAmWVuiZg1afdNSPbtJ8M8r9ArGRT', '/dns/bootnode.1.lightclient.goldberg.avail.tools/udp/37000/quic-v1']]#" config.yaml && \
    cat config.yaml && \
    curl -L https://github.com/availproject/avail-light/releases/download/v1.7.5-rc2/avail-light-linux-amd64.tar.gz --output avail-light-linux-amd64.tar.gz && \
    tar -xf avail-light-linux-amd64.tar.gz

LABEL \
    org.opencontainers.image.authors="The Avail Project Team" \
    org.opencontainers.image.url="https://www.availproject.org/" \
    org.opencontainers.image.documentation="https://github.com/availproject/avail-light#readme" \
    org.opencontainers.image.version="1.4.3" \
    org.opencontainers.image.revision="1" \
    org.opencontainers.image.vendor="The Avail Project" \
    org.opencontainers.image.licenses="MIT" \
    org.opencontainers.image.title="Avail Light Client" \
    org.opencontainers.image.description="Data Availability Docker Node"

CMD ["/bin/sh", "entrypoint.sh"]
