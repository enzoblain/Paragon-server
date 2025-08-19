-- SQL script to create the necessary tables for the trading database

CREATE TABLE symbols (
    id SERIAL PRIMARY KEY,
    symbol TEXT UNIQUE NOT NULL
);

CREATE TABLE candles (
    id SERIAL,
    symbol TEXT NOT NULL,
    timerange TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    open DOUBLE PRECISION NOT NULL,
    high DOUBLE PRECISION NOT NULL,
    close DOUBLE PRECISION NOT NULL,
    low DOUBLE PRECISION NOT NULL,
    volume DOUBLE PRECISION NOT NULL,
    direction TEXT NOT NULL,
    UNIQUE(symbol, timerange, timestamp),
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);
CREATE INDEX ON candles (symbol, timerange, timestamp DESC);

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL,
    label TEXT NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    high DOUBLE PRECISION NOT NULL,
    low DOUBLE PRECISION NOT NULL,
    open DOUBLE PRECISION NOT NULL,
    close DOUBLE PRECISION NOT NULL,
    volume DOUBLE PRECISION NOT NULL,
    UNIQUE(label, start_time),
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);
CREATE INDEX ON sessions (label, start_time DESC);

CREATE TABLE two_d_structures (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL,
    structure TEXT NOT NULL,
    timerange TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    high DOUBLE PRECISION NOT NULL,
    low DOUBLE PRECISION NOT NULL,
    direction TEXT NOT NULL,
    UNIQUE (structure, timerange, timestamp),
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);
CREATE INDEX ON two_d_structures (structure, timerange, timestamp DESC);

CREATE TABLE one_d_structures (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL,
    structure TEXT NOT NULL,
    timerange TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    direction TEXT NOT NULL,
    UNIQUE (structure, timerange, timestamp),
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);
CREATE INDEX ON one_d_structures (structure, timerange, timestamp DESC);

CREATE TABLE trends (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL,
    timerange TEXT NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    direction TEXT NOT NULL,
    high DOUBLE PRECISION NOT NULL,
    low DOUBLE PRECISION NOT NULL,
    UNIQUE(symbol, timerange, start_time),
    FOREIGN KEY (symbol) REFERENCES symbols(symbol)
);
CREATE INDEX ON trends (symbol, timerange, start_time DESC);