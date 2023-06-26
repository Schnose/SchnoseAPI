CREATE TABLE Players (
	id        INT          NOT NULL PRIMARY KEY,
	name      VARCHAR(255) NOT NULL DEFAULT 'unknown',
	is_banned BOOLEAN      NOT NULL DEFAULT FALSE
);

CREATE Table Modes (
	id   SMALLINT     NOT NULL PRIMARY KEY,
	name VARCHAR(255) NOT NULL
);

CREATE TABLE Maps (
	id          SMALLINT     NOT NULL PRIMARY KEY,
	name        VARCHAR(255) NOT NULL,
	ranked      BOOLEAN      NOT NULL DEFAULT FALSE,
	workshop_id INT,
	filesize    BIGINT,
	approved_by INT                   REFERENCES Players (id),
	created_on  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_on  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE Mappers (
	player_id INT      NOT NULL REFERENCES Players (id),
	map_id    SMALLINT NOT NULL REFERENCES Maps (id),

	PRIMARY KEY (player_id, map_id)
);

CREATE TABLE Courses (
	id     INT      NOT NULL PRIMARY KEY,
	map_id SMALLINT NOT NULL REFERENCES Maps (id),
	stage  SMALLINT NOT NULL,
	tier   SMALLINT
);

CREATE TABLE Filters (
	course_id INT      NOT NULL REFERENCES Courses (id),
	mode_id   SMALLINT NOT NULL REFERENCES Modes (id),

	PRIMARY KEY (course_id, mode_id)
);

CREATE TABLE Servers (
	id          SMALLINT     NOT NULL PRIMARY KEY,
	name        VARCHAR(255) NOT NULL,
	owned_by    INT          NOT NULL REFERENCES Players (id)
);

CREATE TABLE Records (
	id         INT       NOT NULL PRIMARY KEY,
	course_id  INT       NOT NULL REFERENCES Courses (id),
	mode_id    SMALLINT  NOT NULL REFERENCES Modes (id),
	player_id  INT       NOT NULL REFERENCES Players (id),
	server_id  SMALLINT  NOT NULL REFERENCES Servers (id),
	time       FLOAT8    NOT NULL,
	teleports  SMALLINT  NOT NULL,
	created_on TIMESTAMP NOT NULL
);

