INSERT INTO users ( username, email, password )
VALUES ( $1, $2, $3 )
RETURNING $table_fields;
