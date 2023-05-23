CREATE TABLE model_types (
  id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  name TEXT NOT NULL,
  category TEXT NOT NULL
);

COMMENT ON COLUMN model_types.category IS 'The purpose of the model';

CREATE TYPE model_status AS ENUM ('downloading', 'training', 'ready', 'error');

CREATE TABLE models (
  id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  model_type INTEGER NOT NULL REFERENCES model_types(id) ON DELETE CASCADE,
  version INTEGER NOT NULL DEFAULT 0,
  status model_status NOT NULL,
  status_message TEXT,
  path TEXT NOT NULL,
  added_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON COLUMN models.path IS 'The location of the model weights and configuration files';

CREATE TABLE sources (
  id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  name TEXT NOT NULL,
  color TEXT
);

COMMENT ON TABLE sources IS 'A source of items, e.g. a directory, a website, etc.';

CREATE TABLE tags (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  name text not null,
  color text
);

CREATE TYPE item_status AS ENUM ('pending', 'processing', 'ready', 'error');

CREATE TABLE items (
  id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,

  status item_status NOT NULL DEFAULT 'pending',

  content_type TEXT NOT NULL,

  external_id TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  hash BYTEA NOT NULL,

  saved_original_path TEXT,
  tags BIGINT[] NOT NULL DEFAULT '{}',

  processed_content TEXT NOT NULL,

  -- Metadata that we may or may not be able to glean from the file
  name TEXT,
  title TEXT,
  author TEXT,
  description TEXT,

  generated_summary TEXT,

  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  -- Set if the user chose to hide this item from the search results
  hidden BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX items_hash ON items (hash);
CREATE INDEX items_tags ON items USING GIN (tags);

COMMENT ON COLUMN items.content_type IS 'The MIME type of the item';
COMMENT ON COLUMN items.external_id IS 'The location where the item was found';
COMMENT ON COLUMN items.processed_content IS 'The content of the item after processing, e.g. extracted text from a PDF';

CREATE TABLE item_chunks (
  item_id BIGINT NOT NULL,
  sequence_num INTEGER NOT NULL,
  model_id INTEGER NOT NULL,
  start_idx INTEGER NOT NULL,
  end_idx INTEGER NOT NULL,
  embedding BYTEA NOT NULL,
  PRIMARY KEY (model_id, item_id, sequence_num)
);

COMMENT ON COLUMN item_chunks.sequence_num IS 'Which numbered chunk this is in the document, starting from 0';
COMMENT ON COLUMN item_chunks.start_idx IS 'The index of the first character of the chunk in the document';
COMMENT ON COLUMN item_chunks.end_idx IS 'The index of the last character of the chunk in the document';


