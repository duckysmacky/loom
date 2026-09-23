-- Where the user last dragged the node on the board canvas. Both NULL means
-- "never placed" - the frontend auto-lays those nodes out.
ALTER TABLE nodes
    ADD COLUMN canvas_x double precision,
    ADD COLUMN canvas_y double precision,
    ADD CONSTRAINT nodes_canvas_position_pair
        CHECK ((canvas_x IS NULL) = (canvas_y IS NULL));
