-- Your SQL goes here;
CREATE TABLE users (
    uid SERIAL PRIMARY KEY,
    UserName VARCHAR NOT NULL,
    Password TEXT NOT NULL
);
CREATE UNIQUE INDEX users_username_key ON users (username);

