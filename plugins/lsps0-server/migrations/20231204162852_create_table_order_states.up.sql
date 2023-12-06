-- Table containing the state of lsps1 orders
-- Whenever the state is updated a new row is added
-- The current_state is the row for which `created_at` is the highest
create TABLE lsps1_order_state(
  id INTEGER PRIMARY KEY NOT NULL,
  order_id INTEGER UNIQUE NOT NULL,
  order_state_enum_id INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (order_id) REFERENCES lsps1_order(id)
  FOREIGN KEY (order_state_enum_id) REFERENCES lsps1_order_state_enum(id)
);

-- This table contains the 3 possible order_states
-- CREATED, COMPLETED, FAILED
CREATE TABLE lsps1_order_state_enum(
  id INTEGER PRIMARY KEY NOT NULL,
  order_state TEXT NOT NULL UNIQUE
);

INSERT INTO lsps1_order_state_enum
  (id, order_state)
VALUES
  (1, "CREATED"),
  (2, "COMPLETED"),
  (3, "FAILED");


