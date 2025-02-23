# log4ham

An application to allow capture of Ham contacts.

# Version 1

Create an abaility to log Amateur Radio calls

| Date | Start Time | End Time | Frequency (Hz) | Mode | Power (dBW) | Station (called/worked) |

# Pull Secrets with GHCR

Follow this guide: https://dev.to/asizikov/using-github-container-registry-with-kubernetes-38fb

Pull your DB to local to test

    kubectl -n dbs port-forward svc/postgresql-postgresql-ha-pgpool 5432

    cargo watch -x "run start --config test-data/config-be.yaml --secrets ${PWD}/test-data/secrets"


# ToDo

* [ ] Swap nginx image to alpine version to reduce size
* [ ]
