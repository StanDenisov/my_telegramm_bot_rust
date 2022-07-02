CREATE TABLE IF NOT EXISTS link_message
(
    id          BIGSERIAL PRIMARY KEY,
    text        VARCHAR(256)    NOT NULL,
    chat_id     BIGSERIAL NOT NULL,
    message_id  BIGSERIAL NOT NULL
);

CREATE TABLE IF NOT EXISTS update
(
    id    BIGSERIAL PRIMARY KEY,
    update_id BIGSERIAL
);

CREATE TABLE IF NOT EXISTS message
(
    message_id  BIGSERIAL PRIMARY KEY,
    text        VARCHAR(256)    NOT NULL,
    chat_id     BIGSERIAL NOT NULL
);

INSERT INTO update VALUES (1, 1)
