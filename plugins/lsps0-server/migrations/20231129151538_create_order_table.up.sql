-- The table containing all orders
CREATE TABLE lsps1_order (
  uuid TEXT PRIMARY KEY NOT NULL,			
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

-- This table contains all order_states
-- If the order state is updated a new entry is added to this table
create TABLE lsps1_order_state(
  id INTEGER PRIMARY_KEY,
  order_id TEXT,
  order_state_enum_id INTEGER,
  created_at INTEGER
);

-- This table contains the 3 possible order_states
-- CREATED, COMPLETED, FAILED
CREATE TABLE lsps1_order_state_enum(
  id INTEGER PRIMARY KEY NOT NULL,
  order_state TEXT NOT NULL
);

INSERT INTO lsps1_order_state_enum
  (id, order_state)
VALUES
  (1, "CREATED"),
  (2, "COMPLETED"),
  (3, "FAILED");
