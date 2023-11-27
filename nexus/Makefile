.PHONY: start start-existing start-new build-docker-image

# Define the NFT variable with a default value of true
NFT ?= true

CONTAINER_NAME_NFT=avail-light-nft
IMAGE_NAME_NFT=avail-light-nft
CONTAINER_NAME_PAYMENTS=avail-light-payments
IMAGE_NAME_PAYMENTS=avail-light-payments
TEST_PRIVATE_KEY_PATH=../../demo-stf/src/sov-cli/test_data/minter_private_key.json
APP_ID_NFT=7
APP_ID_PAYMENTS=8
LIGHT_CLIENT_PORT_NFT=8000
LIGHT_CLIENT_PORT_PAYMENTS=8001
LIBP2P_PORT_NFT=37000
LIBP2P_PORT_PAYMENTS=37001

check-docker:
	@command -v docker > /dev/null 2>&1 || { echo "Docker is not installed"; exit 1; }

check-container-running:
	@echo "Container $(CONTAINER_NAME) running"
	@docker ps --format '{{.Names}}' | grep -w $(CONTAINER_NAME) > /dev/null 2>&1

check-container-exists:
	@echo "Container $(CONTAINER_NAME) exists"
	@docker ps -a --format '{{.Names}}' | grep -w $(CONTAINER_NAME) > /dev/null 2>&1

build-docker-image:
	@echo "Starting NFT light client with APP_ID=$(APP_ID)"
	@docker build --platform linux/amd64 --build-arg NODE_CLIENT_WS=wss://goldberg.avail.tools:443/ws --build-arg NODE_CLIENT_RPC=https://goldberg.avail.tools/rpc   --build-arg APP_ID=$(APP_ID) --build-arg LIBP2P_PORT=$(LIBP2P_PORT) -t $(IMAGE_NAME) .

check-image-exists:
	@if docker images $(IMAGE_NAME) | awk '{print $$1}' | grep -q $(IMAGE_NAME); then \
		echo "Image $(IMAGE_NAME) exists."; \
	else \
		$(MAKE) build-docker-image; \
	fi

start-existing:
	@echo "Resuming existing container: $(CONTAINER_NAME)"
	docker start $(CONTAINER_NAME)

start-new:
	@echo "Starting new container: $(CONTAINER_NAME)"
	docker run -d --platform linux/amd64 -p $(LIGHT_CLIENT_PORT):7000 --name $(CONTAINER_NAME) -t --restart unless-stopped $(IMAGE_NAME)

start-container: check-image-exists
	@$(MAKE) check-container-running || { $(MAKE) check-container-exists && $(MAKE) start-existing || $(MAKE) start-new; }

start: check-docker
ifeq ($(NFT),true)
	LIGHT_CLIENT_PORT=$(LIGHT_CLIENT_PORT_NFT) CONTAINER_NAME=$(CONTAINER_NAME_NFT) IMAGE_NAME=$(IMAGE_NAME_NFT) APP_ID=$(APP_ID_NFT) LIBP2P_PORT=$(LIBP2P_PORT_NFT)  $(MAKE) start-container
else
	LIGHT_CLIENT_PORT=$(LIGHT_CLIENT_PORT_PAYMENTS) CONTAINER_NAME=$(CONTAINER_NAME_PAYMENTS) IMAGE_NAME=$(IMAGE_NAME_PAYMENTS) APP_ID=$(APP_ID_PAYMENTS) LIBP2P_PORT=$(LIBP2P_PORT_PAYMENTS) $(MAKE) start-container
endif

stop: check-docker
ifeq ($(NFT),true)
	LIGHT_CLIENT_PORT=$(LIGHT_CLIENT_PORT_NFT) CONTAINER_NAME=$(CONTAINER_NAME_NFT) $(MAKE) start-container
	docker stop $(CONTAINER_NAME_NFT)
else
	LIGHT_CLIENT_PORT=$(LIGHT_CLIENT_PORT_PAYMENTS) CONTAINER_NAME=$(CONTAINER_NAME_PAYMENTS) $(MAKE) start-container
	docker stop $(CONTAINER_NAME_PAYMENTS)
endif

clean: check-docker
ifeq ($(NFT),true)
	LIGHT_CLIENT_PORT=$(LIGHT_CLIENT_PORT_NFT) CONTAINER_NAME=$(CONTAINER_NAME_NFT) $(MAKE) start-container
	docker stop $(CONTAINER_NAME_NFT)
	docker rm $(CONTAINER_NAME_NFT)
else
	LIGHT_CLIENT_PORT=$(LIGHT_CLIENT_PORT_PAYMENTS) CONTAINER_NAME=$(CONTAINER_NAME_PAYMENTS) $(MAKE) start-container
	docker stop $(CONTAINER_NAME_PAYMENTS)
	docker rm $(CONTAINER_NAME_PAYMENTS)
endif
