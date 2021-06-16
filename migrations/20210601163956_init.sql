-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE delta (
  id UUID DEFAULT uuid_generate_v4(),
  ty varchar(255) NOT NULL,
  body jsonb NOT NULL,
  author varchar(255) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE TABLE projection (
  id UUID PRIMARY KEY UNIQUE DEFAULT uuid_generate_v4(),
  ty varchar(255) NOT NULL,
  body jsonb NOT NULL,
  last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
)