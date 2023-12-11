CREATE TABLE lsps1_channel (
  id INTEGER PRIMARY KEY NOT NULL,
  order_id INTEGER NOT NULL UNIQUE,
  channel_id TEXT NOT NULL UNIQUE,
  FOREIGN KEY (order_id) references lsps1_order(id)
);
