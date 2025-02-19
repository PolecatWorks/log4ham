IMAGE_NAME=log4ham
TAG ?= 0.1.0

ifeq ($(shell command -v podman 2> /dev/null),)
    DOCKER?=docker
else
    DOCKER?=podman
endif

DOCKER=docker

.PHONY: build crds

all:
	@echo making everything

.ONESHELL:

ghcr-login:
	echo ${GHCR_TOKEN} | $(DOCKER) login ghcr.io -u $(GHCR_USER) --password-stdin


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

PG_CONTAINER=test-postgres
PG_ADMIN_PASSWORD ?= adminpw1
PG_DATABASE=mydb
PG_USERNAME=myuser
PG_PASSWORD=userpw1


postgres-stop:
	podman stop $(PG_CONTAINER)
	podman container rm $(PG_CONTAINER)


postgres-start:
	podman run --name $(PG_CONTAINER) -p 5432:5432 -e POSTGRES_PASSWORD=$(PG_ADMIN_PASSWORD) -d postgres


postgres-db: export PGPASSWORD=$(PG_ADMIN_PASSWORD)
postgres-db:
	psql -h localhost -U postgres postgres -c "create database $(PG_DATABASE);" || true
	psql -h localhost -U postgres postgres -c "create user $(PG_USERNAME) with encrypted password '$(PG_PASSWORD)';grant all privileges on database $(PG_DATABASE) to $(PG_USERNAME);" || true
	psql -h localhost -U postgres $(PG_DATABASE) -c "create SCHEMA $(PG_USERNAME) AUTHORIZATION $(PG_USERNAME);"


postgres-schema:
	sqlx migrate run --database-url postgres://$(PG_USERNAME):$(PG_PASSWORD)@localhost/$(PG_DATABASE)

postgres-config: postgres-db postgres-schema


watch: export DATABASE_URL=postgres://postgres:mypw@localhost/postgres
watch:
	cargo watch --ignore test_data -x "test ${TEST} -- --nocapture"


bench: export DATABASE_URL=postgres://myuser:mypass@localhost/mydb
bench:
	cargo bench Handlers
	open target/criterion/report/index.html

watch-db-check:
	cargo watch -x "run -- db-check --config test_data/myconfig.yaml"

watch-run:
	cargo watch -x "run -- receive --config test_data/myconfig.yaml"


# Run the container
docker-fe-dev:
	$(DOCKER) build . -t $(IMAGE_NAME)-dev -f Dockerfile --target dev
	$(DOCKER) image ls $(IMAGE_NAME)-dev

docker-fe-dev-run: docker-dev
	$(DOCKER) run --rm -it --mount type=bind,source=$(PWD),target=/app $(IMAGE_NAME)-dev
