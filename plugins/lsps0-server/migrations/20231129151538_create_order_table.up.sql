
CREATE TABLE lsps1_order (
  id INTEGER PRIMARY KEY NOT NULL,
  uuid TEXT NOT NULL UNIQUE,			   
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

