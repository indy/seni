## Usage

Copy .env-example and rename it to .env
Update the username and password details in the .env file's DATABASE_URL

export the values of the .env file into a shell.
```
export $(egrep -v '^#' .env | xargs)
```

Create the postgres database with the same details as given for DATABASE_URL.

```
createdb -U postgres seni-dev
```

Load the database schema.

```
psql -d "$DATABASE_URL" -f ./db/schema.psql
```
