IMAGE_NAME=log4ham
TAG ?= 0.1.0

ifeq ($(shell command -v podman 2> /dev/null),)
    DOCKER?=docker
else
    DOCKER?=podman
endif

DOCKER=docker

FE_DIR=container-fe
BE_DIR=container-be

.PHONY: build crds

all:
	@echo making everything

.ONESHELL:

ghcr-login:
	echo ${GHCR_TOKEN} | $(DOCKER) login ghcr.io -u $(GHCR_USER) --password-stdin

	docker-be-run: docker-be
		$(DOCKER) run --rm -it -p 8081:8081 $(IMAGE_NAME)-be

	pg-schema-info:
		@cd ${BE_DIR} && sqlx migrate info --database-url postgres://${PG_USER}:${PGPASSWORD}@localhost/${PG_NAME}

	pg-schema-run:
		@cd ${BE_DIR} && sqlx migrate run --database-url postgres://${PG_USER}:${PGPASSWORD}@localhost/${PG_NAME}

	pg-schema-revert:
		@cd ${BE_DIR} && sqlx migrate revert --database-url postgres://${PG_USER}:${PGPASSWORD}@localhost/${PG_NAME}
docker-fe:
	{ \
	$(DOCKER) build container-fe -t $(IMAGE_NAME)-fe -f container-fe/Dockerfile; \
	$(DOCKER) image ls $(IMAGE_NAME)-fe; \
	}

docker-fe-run: docker-fe
	$(DOCKER) run --rm -it -p 8080:8080 $(IMAGE_NAME)-fe

docker-be: PKG_NAME=log4ham
docker-be:
	{ \
	$(DOCKER) build container-be -t $(IMAGE_NAME)-be -f container-be/Dockerfile --build-arg PKG_NAME=${PKG_NAME}; \
	$(DOCKER) image ls $(IMAGE_NAME)-be; \
	}


cargo-build:
	$(MAKE) -C container-be cargo-build

cargo-doc:
	@cargo doc --no-deps --document-private-items
	@open target/doc/largejson/index.html

pg-pool-restart:
	kubectl -n dbs rollout restart deployment postgresql-postgresql-ha-pgpool
	kubectl -n dbs get pods -w

# port-forward the postgres service
pg-port-forward:
	kubectl -n dbs port-forward svc/postgresql-postgresql-ha-pgpool 5432:5432


PG_SECRET=log4ham-pg
PG_USER=$(shell kubectl -n log4ham get secret $(PG_SECRET) -o jsonpath="{.data.DB_USER}" | base64 --decode)
PGPASSWORD=$(shell kubectl -n log4ham get secret $(PG_SECRET) -o jsonpath="{.data.DB_PASS}" | base64 --decode)
PG_NAME=$(shell kubectl -n log4ham get secret $(PG_SECRET) -o jsonpath="{.data.DB_NAME}" | base64 --decode)
DATABASE_URL=postgres://${PG_USER}:${PGPASSWORD}@localhost/${PG_NAME}

secret-update:
	kubectl -n log4ham get secrets log4ham-pg -o jsonpath="{.data.DB_PASS}" | base64 -d > $(BE_DIR)/test-data/secrets/db/password
	kubectl -n log4ham get secrets log4ham-pg -o jsonpath="{.data.DB_USER}" | base64 -d > $(BE_DIR)/test-data/secrets/db/username

pg-login:
	@PGPASSWORD=${PGPASSWORD} psql -h localhost -U ${PG_USER} -d ${PG_NAME}

pg-connection:
	@echo postgres://${PG_USER}:${PGPASSWORD}@localhost/${PG_NAME}

pg-schema-info:
	@cd container-be && sqlx migrate info --database-url postgres://${PG_USER}:${PGPASSWORD}@localhost/${PG_NAME}

pg-schema-run:
	@cd container-be && sqlx migrate run --database-url postgres://${PG_USER}:${PGPASSWORD}@localhost/${PG_NAME}

pg-schema-revert:
	@cd container-be && sqlx migrate revert --database-url postgres://${PG_USER}:${PGPASSWORD}@localhost/${PG_NAME}

pg-test-container:
	@kubectl delete pod pg-test-pod || true
	@kubectl run -it --rm pg-test-pod --image=postgres:17.4 --env="POSTGRES_USER=${PG_USER}" --env="POSTGRES_PASSWORD=${PGPASSWORD}" --env="POSTGRES_DB=${PG_NAME}" --port=5432

pg-docker-test-container:
	@docker rm -f pg-test-container || true
	@docker run -it --rm --name pg-test-container -e POSTGRES_USER=${PG_USER} -e POSTGRES_PASSWORD=${PGPASSWORD} -e POSTGRES_DB=${PG_NAME} -p 5432:5432 postgres:17.4

pg-test-forward:
	kubectl port-forward pod/pg-test-pod 5432:5432

criterion: export DATABASE_URL=postgres://${PG_USER}:${PGPASSWORD}@localhost/${PG_NAME}
criterion:
	@cd container-be && cargo criterion
	open container-be/target/criterion/report/index.html

watch-db-check:
	cd ${BE_DIR} && cargo watch -x "run -- db-check --config test-data/config-localhost.yaml --secrets test-data/secrets"

watch-config-check:
	cd ${BE_DIR} && cargo watch -x "run -- config-check --config test-data/config-localhost.yaml --secrets test-data/secrets"

watch-test:
	cd ${BE_DIR} && DATABASE_URL=${DATABASE_URL} cargo watch --ignore test_data -x "test"
# "test stationsetup"
#  --nocapture

watch-run:
	cd ${BE_DIR} && DATABASE_URL=${DATABASE_URL} cargo watch -x "run -- start --config test-data/config-localhost.yaml --secrets test-data/secrets --automigrate"

watch-serve:
	cd ${FE_DIR} && ng serve


# Run the container
docker-fe-dev:
	$(DOCKER) build . -t $(IMAGE_NAME)-dev -f Dockerfile --target dev
	$(DOCKER) image ls $(IMAGE_NAME)-dev

docker-fe-dev-run: docker-dev
	$(DOCKER) run --rm -it --mount type=bind,source=$(PWD),target=/app $(IMAGE_NAME)-dev
