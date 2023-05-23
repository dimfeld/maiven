#!/bin/bash
set -euox pipefail
ENV_FILE=$(dirname $(realpath $0))/../.env

source $ENV_FILE

DATABASE_PASSWORD="${DATABASE_PASSWORD:-$(openssl rand -base64 32)}"
DATABASE_USER=${DATABASE_USER:-maiven}

# A database URL that works to connect as a superuser.
# If running as the `postgres` user and creating a local database then this probably works.
DATABASE_ADMIN_URL=${DATABASE_ADMIN_URL:-postgres}

psql -e -d ${DATABASE_ADMIN_URL} <<EOF
  CREATE ROLE ${DATABASE_USER} WITH
      INHERIT LOGIN CREATEDB
      PASSWORD '${DATABASE_PASSWORD}';
EOF

echo Created database user $DATABASE_USER with password $DATABASE_PASSWORD
