-- There are no actual FK contraints here because Planetscale does not support them.

CREATE TABLE IF NOT EXISTS modes (
	id   TINYINT UNSIGNED NOT NULL,
	name VARCHAR(255)     NOT NULL,

	PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS players (
	id        INT UNSIGNED NOT NULL,
	name      VARCHAR(255) NOT NULL,
	is_banned BOOLEAN      NOT NULL DEFAULT FALSE,

	PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS maps (
	id          SMALLINT     UNSIGNED NOT NULL,
	name        VARCHAR(255)          NOT NULL,
	global      BOOLEAN               NOT NULL DEFAULT FALSE,
	filesize    INT          UNSIGNED NOT NULL,
	approved_by INT          UNSIGNED NOT NULL,
	created_on  TIMESTAMP             NOT NULL,
	updated_on  TIMESTAMP             NOT NULL,

	PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS mappers (
	-- REFERENCES maps (id)
	map_id    SMALLINT UNSIGNED NOT NULL,
	-- REFERENCES players (id)
	mapper_id INT      UNSIGNED NOT NULL
);

CREATE TABLE IF NOT EXISTS courses (
	id     INT      UNSIGNED NOT NULL,
	-- REFERENCES maps (id)
	map_id SMALLINT UNSIGNED NOT NULL,
	stage  TINYINT  UNSIGNED NOT NULL,
	tier   TINYINT  UNSIGNED NOT NULL,
	
	PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS filters (
	-- REFERENCES courses (id)
	course_id INT     UNSIGNED NOT NULL,
	-- REFERENCES modes (id)
	mode_id   TINYINT UNSIGNED NOT NULL
);

CREATE TABLE IF NOT EXISTS servers (
	id          SMALLINT     UNSIGNED NOT NULL,
	name        VARCHAR(255)          NOT NULL,
	-- REFERENCES players (id)
	owned_by    INT          UNSIGNED NOT NULL,
	-- REFERENCES players (id)
	approved_by INT          UNSIGNED NOT NULL,

	PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS records (
	id         INT      UNSIGNED NOT NULL,
	-- REFERENCES courses (id)
	course_id  INT      UNSIGNED NOT NULL,
	-- REFERENCES modes (id)
	mode_id    INT      UNSIGNED NOT NULL,
	-- REFERENCES players (id)
	player_id  INT      UNSIGNED NOT NULL,
	-- REFERENCES servers (id)
	server_id  INT      UNSIGNED NOT NULL,
	time       DOUBLE            NOT NULL,
	teleports  SMALLINT UNSIGNED NOT NULL,
	created_on TIMESTAMP         NOT NULL,

	PRIMARY KEY (id)
);
