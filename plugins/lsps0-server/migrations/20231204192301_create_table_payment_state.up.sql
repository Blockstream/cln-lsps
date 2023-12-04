CREATE TABLE lsps1_payment_state (
  id INTEGER PRIMARY KEY NOT NULL,
  payment_details_id INTEGER UNIQUE NOT NULL,
  payment_state INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (payment_details_id) REFERENCES lsps1_payment_details(id)
);

CREATE TABLE lsps1_payment_state_enum (
  id INTEGER PRIMARY KEY NOT NULL,
  payment_state TEXT UNIQUE NOT NULL
);

INSERT INTO lsps1_payment_state_enum 
  (id, payment_state)
VALUES
  (1, "EXPECT_PAYMENT"),
  (2, "HOLD"),
  (3, "PAID"),
  (4, "REFUNDED");
