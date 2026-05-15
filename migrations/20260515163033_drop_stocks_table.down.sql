CREATE TABLE stocks (
    id SERIAL PRIMARY KEY,
    code VARCHAR(10) NOT NULL UNIQUE,
    name TEXT NOT NULL,
    closing_price DECIMAL(10,2) NOT NULL,
    monthly_average_price DECIMAL(10,2) NOT NULL
);
