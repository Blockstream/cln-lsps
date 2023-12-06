-- This table contains the lsps1 payment details
-- The rows in this table should never be updated
CREATE TABLE lsps1_payment_details (
  id INTEGER PRIMARY KEY NOT NULL,
  order_id TEXT NOT NULL UNIQUE,
  fee_total_sat INTEGER NOT NULL,
  order_total_sat INTEGER NOT NULL,
  bolt11_invoice TEXT NOT NULL,
  bolt11_invoice_label TEXT NOT NULL,   -- This label is used by core lightning 
  onchain_address TEXT,
  onchain_block_confirmations_required INTEGER,
  minimum_fee_for_0conf INTEGER,
  FOREIGN KEY (order_id) REFERENCES lsps1_order(id)
);


