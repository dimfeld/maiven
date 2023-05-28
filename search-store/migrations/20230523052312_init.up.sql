CREATE TYPE model_status AS ENUM ('uninitialized', 'downloading', 'training', 'ready', 'error');

CREATE TABLE models (
  id INTEGER PRIMARY KEY GENERATED BY DEFAULT AS IDENTITY,
  name TEXT NOT NULL,
  category TEXT NOT NULL,
  status model_status NOT NULL DEFAULT 'uninitialized',
  status_message TEXT,
  params jsonb NOT NULL,
  added_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON COLUMN models.category IS 'The purpose of this model';
COMMENT ON COLUMN models.params IS 'Parameters indicating how to run this model';

CREATE TABLE sources (
  id INTEGER PRIMARY KEY GENERATED BY DEFAULT AS IDENTITY,
  name TEXT NOT NULL,
  color TEXT
);

COMMENT ON TABLE sources IS 'A source of items, e.g. a directory, a website, etc.';

CREATE TABLE tags (
  id INTEGER PRIMARY KEY GENERATED BY DEFAULT AS IDENTITY,
  name text not null,
  color text
);

CREATE TYPE item_status AS ENUM ('pending', 'processing', 'ready', 'error');

CREATE TABLE items (
  id BIGINT PRIMARY KEY GENERATED BY DEFAULT AS IDENTITY,
  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,

  status item_status NOT NULL DEFAULT 'pending',

  content_type TEXT NOT NULL,

  external_id TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 0,
  hash BYTEA NOT NULL,

  saved_original_path TEXT,
  tags INTEGER[] NOT NULL DEFAULT '{}',

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
  item_id BIGINT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  sequence_num INTEGER NOT NULL,
  model_id INTEGER NOT NULL REFERENCES models(id) ON DELETE CASCADE,
  start_idx INTEGER NOT NULL,
  end_idx INTEGER NOT NULL,
  embedding BYTEA NOT NULL,
  PRIMARY KEY (model_id, item_id, sequence_num)
);

COMMENT ON COLUMN item_chunks.sequence_num IS 'Which numbered chunk this is in the document, starting from 0';
COMMENT ON COLUMN item_chunks.start_idx IS 'The index of the first character of the chunk in the document';
COMMENT ON COLUMN item_chunks.end_idx IS 'The index of the last character of the chunk in the document';

CREATE TABLE chat_system_messages (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  name TEXT,
  message TEXT NOT NULL,
  hidden BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE chat_sessions (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  name TEXT,
  system_message_id BIGINT REFERENCES chat_system_messages(id) ON DELETE SET NULL,
  parent_session BIGINT REFERENCES chat_sessions(id) ON DELETE SET NULL,
  hidden BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON COLUMN chat_sessions.system_message_id IS 'The system message for this chat session';
COMMENT ON COLUMN chat_sessions.parent_session IS 'If this chat session branched from a message in another session, this is the parent session.';

CREATE TABLE chat_summaries (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  parent_id BIGINT REFERENCES chat_summaries(id) ON DELETE CASCADE,
  session_id BIGINT NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
  summary TEXT NOT NULL,
  start_message_id BIGINT NOT NULL,
  end_message_id BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE chat_messages (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  session_id BIGINT NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
  parent_id BIGINT REFERENCES chat_messages(id) ON DELETE CASCADE,
  important BOOLEAN NOT NULL DEFAULT FALSE,
  user_message TEXT NOT NULL,
  ai_message TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON COLUMN chat_messages.important IS 'If this message is marked as important and should be given preference as the context growws longer.';

