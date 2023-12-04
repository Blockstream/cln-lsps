WITH to_delete as (SELECT id from lsps1_order WHERE uuid = ?0)
DELETE FROM lsps1_order_state where id in to_delete;

