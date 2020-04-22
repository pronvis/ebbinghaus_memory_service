CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  email VARCHAR UNIQUE NOT NULL
);
CREATE TABLE memories (
  id SERIAL PRIMARY KEY,
  user_id INT references users(id) NOT NULL,
  topic VARCHAR,
  text TEXT NOT NULL
);
CREATE TABLE phases (
  id SERIAL PRIMARY KEY,
  phase_number INT UNIQUE NOT NULL,
  seconds_to_wait BIGINT NOT NULL
);
CREATE TABLE schedules (
  id SERIAL PRIMARY KEY,
  memory_id INT references memories(id) NOT NULL,
  phase_number INT references phases(phase_number) NOT NULL,
  next_run BIGINT
);

INSERT INTO phases(phase_number, seconds_to_wait) VALUES(1, 0);
INSERT INTO phases(phase_number, seconds_to_wait) VALUES(2, 15*60);
INSERT INTO phases(phase_number, seconds_to_wait) VALUES(3, 10*60*60);
INSERT INTO phases(phase_number, seconds_to_wait) VALUES(4, 28*60*60);
INSERT INTO phases(phase_number, seconds_to_wait) VALUES(5, 4*24*60*60);
INSERT INTO phases(phase_number, seconds_to_wait) VALUES(6, 30*24*60*60);
INSERT INTO phases(phase_number, seconds_to_wait) VALUES(7, 4*30*24*60*60);
