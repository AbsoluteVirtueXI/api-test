CREATE TABLE IF NOT EXISTS todos(
    id SERIAL PRIMARY KEY NOT NULL,
    text TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE
);
