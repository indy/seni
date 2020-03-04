# Curio

## Usage

Rename the .env-example file to .env and update the Postgres details.

export the values of the .env file into a shell.
```
export $(egrep -v '^#' .env | xargs)
```

Create the postgres database.

```
createdb -U postgres $POSTGRES_DB
```

Load the database schema.

```
psql -d "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST/$POSTGRES_DB" -f ./db/schema.psql
```

Run.

```
cargo run
```
