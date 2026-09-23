-- updated_at is set explicitly by the repository layer on every UPDATE, not triggered.
CREATE TABLE topics (
    id uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name text NOT NULL,
    color text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT topics_unique_user_name UNIQUE (user_id, name)
);

CREATE TABLE node_topics (
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    topic_id uuid NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    PRIMARY KEY (node_id, topic_id)
);

CREATE INDEX idx_node_topics_topic ON node_topics (topic_id);
