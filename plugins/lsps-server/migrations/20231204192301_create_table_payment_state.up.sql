CREATE TABLE lsps1_payment_state (
  id INTEGER PRIMARY KEY NOT NULL,
  payment_details_id INTEGER NOT NULL,
  payment_state INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  generation INTEGER NOT NULL,
  FOREIGN KEY (payment_details_id) REFERENCES lsps1_payment_details(id),
  FOREIGN KEY (payment_state) references lsps1_payment_state_enum(id)
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
