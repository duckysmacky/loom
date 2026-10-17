-- Which side of each end an annotation arrow attaches to.
CREATE TYPE canvas_side AS ENUM ('top', 'right', 'bottom', 'left');

ALTER TABLE canvas_arrows
    ADD COLUMN from_side canvas_side NOT NULL DEFAULT 'right',
    ADD COLUMN to_side canvas_side NOT NULL DEFAULT 'left';
