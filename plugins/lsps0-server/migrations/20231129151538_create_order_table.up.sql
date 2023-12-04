-- The table containing all orders
CREATE TABLE lsps1_order (
  id INTEGER PRIMARY KEY NOT NULL,
  uuid TEXT NOT NULL,			   
  client_node_id TEXT NOT NULL,				-- The node-id of the client
  lsp_balance_sat INTEGER NOT NULL,			-- as requested by the client
  client_balance_sat INTEGER NOT NULL,			-- as requested by the client
  confirms_within_blocks INTEGER NOT NULL,		-- as requested by the client
  channel_expiry_blocks INTEGER NOT NULL,		-- as requested by the client
  token TEXT,	 					-- as requested by the client
  refund_onchain_address TEXT,				-- as requested by the client
  announce_channel BOOLEAN NOT NULL,			-- as requested by the client
  created_at INTEGER NOT NULL,				-- timestamp: seconds since UNIX epoch in UTC
  expires_at INTEGER NOT NULL 				-- timestamp: seconds since UNIX epoch in UTC
);

CREATE INDEX lsps1_order_uuid_index ON lsps1_order(uuid, client_node_id);

-- This table contains all order_states
-- If the order state is updated a new entry is added to this table
create TABLE lsps1_order_state(
  id INTEGER PRIMARY KEY NOT NULL,
  order_id INTEGER,
  order_state_enum_id INTEGER,
  created_at INTEGER,
  FOREIGN KEY (order_id) REFERENCES lsps1_order(id)
  FOREIGN KEY (order_state_enum_id) REFERENCES lsps1_order_state_enum(id)
);

-- This table contains the 3 possible order_states
-- CREATED, COMPLETED, FAILED
CREATE TABLE lsps1_order_state_enum(
  id INTEGER PRIMARY KEY NOT NULL,
  order_state TEXT NOT NULL
);

-- This table contains the lsps1 payment details
-- The table will only contain parameters that remain
-- unmodified
CREATE TABLE lsps1_payment_details (
  id INTEGER PRIMARY KEY NOT NULL,
  order_id TEXT NOT NULL,
  fee_total_sat INTEGER NOT NULL,
  order_total_sat INTEGER NOT NULL,
  bolt11_invoice TEXT NOT NULL,
  onchain_address TEXT,
  onchain_block_confirmations_required INTEGER,
  minimum_fee_for_0conf INTEGER,
  FOREIGN KEY (order_id) REFERENCES lsps1_order(id)
);

INSERT INTO lsps1_order_state_enum
  (id, order_state)
VALUES
  (1, "CREATED"),
  (2, "COMPLETED"),
  (3, "FAILED");
