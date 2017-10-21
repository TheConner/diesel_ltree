CREATE EXTENSION IF NOT EXISTS ltree;

CREATE TABLE my_tree (
       id SERIAL PRIMARY KEY,
       path ltree NOT NULL
);

INSERT INTO my_tree (path) VALUES
       ('root'),
       ('root.bacteria'),
       ('root.bacteria.aquificae'),
       ('root.bacteria.thermotogae'),
       ('root.archaea'),
       ('root.archeae.thermoprotei'),
       ('root.archaea.thermoprotei.pyrodictiaceae'),
       ('root.archaea.thermoprotei.thermoproteaceae'),
       ('root.eukaryota'),
       ('root.eukaryota.plantae'),
       ('root.eukaryota.plantae.nematophyta'),
       ('root.eukaryota.plantae.chlorophyta'),
       ('root.eukaryota.animalia'),
       ('root.eukaryota.animalia.chancelloriidae'),
       ('root.eukaryota.animalia.cloudinidae');
