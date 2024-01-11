CREATE TABLE lsps1_channel (
  id INTEGER PRIMARY KEY NOT NULL,
  order_id INTEGER NOT NULL UNIQUE,
  funding_tx TEXT NOT NULL,
  outnum INTEGER NOT NULL,
  FOREIGN KEY (order_id) references lsps1_order(id)
);
