CREATE TABLE responses (
  id INTEGER PRIMARY KEY ASC NOT NULL,
  status SMALLINT NOT NULL,
  request TEXT NOT NULL,
  body TEXT NOT NULL
);

CREATE UNIQUE INDEX responses_requests_index ON responses (request);
