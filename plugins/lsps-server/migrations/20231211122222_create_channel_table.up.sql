CREATE TABLE lsps1_channel (
  order_id INTEGER PRIMARY KEY NOT NULL UNIQUE,
  funding_txid TEXT NOT NULL,
  outnum INTEGER NOT NULL,
  funded_at INTEGER NOT NULL,
  FOREIGN KEY (order_id) references lsps1_order(id)
);
