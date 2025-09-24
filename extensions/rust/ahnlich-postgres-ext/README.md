# Ahnlich Postgres extension (beta)

## Setup instructions

1. Go to: <https://github.com/pgcentralfoundation/pgrx?tab=readme-ov-file#system-requirements> & <https://github.com/pgcentralfoundation/pgrx?tab=readme-ov-file#getting-started> and follow the instructions to install pgrx on your local machine

## Run instructions

```sh
cargo pgrx run
```

When the `psql` shell opens, now you can run:

```sql
CREATE EXTENSION ahnlich_postgres_ext;
```

Then you can run any of the functions defined in lib.rs

```sql
SELECT run_query_with_args(2992);
```
