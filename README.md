# Payments Engine

A simple toy payments engine that processes a series of transactions from a CSV file, updates client accounts, handles disputes and chargebacks, and outputs the final state of clients' accounts as a CSV.

## How to Run

To run the application, use the following cargo command, passing the input CSV file as an argument. [cite_start]The output will be printed to `stdout`[cite: 31, 32].

```bash
cargo run -- transactions.csv > accounts.csv
```

To run the unit tests:

```bash
cargo test
```

To run the stress test:

```bash
rustc generate_csv.rs && ./generate_csv
```

and then:

```bash
cargo run -- million_rows.csv > accounts.csv
```

## Assumptions & Design Decisions

When building this engine, I made the following assumptions based on standard financial/banking logic:

1. **Disputing Withdrawals:** I assumed that only `Deposit` transactions can be disputed (e.g., a client reversing a fiat deposit via their credit card provider). Therefore, only deposits are saved into the `transactions_history`. Withdrawals are processed but not tracked for future disputes.

2. **Locked Accounts:** If an account is frozen due to a `Chargeback`, it immediately rejects any new `Deposit`, `Withdrawal`, or `Dispute` requests. However, the account **will still process** `Resolve` and `Chargeback` events for other transactions that were already under dispute before the lock occurred.

3. **Data Streaming vs Memory:** The application streams the CSV input using `csv::Reader::deserialize`, which is memory-efficient. However, `transactions_history` is kept in memory using a `HashMap`. For a production environment with millions of transactions and thousands of concurrent TCP streams, this history would ideally be stored in a database (e.g., PostgreSQL) or an in-memory datastore with TTL (e.g., Redis).

4. **Error Handling:** If a CSV row is malformed or missing the `amount` field for deposits/withdrawals, the program silently ignores the record and continues processing. This prevents the entire stream from crashing due to a single bad record.

5. **Precision:** Used the `rust_decimal` crate to guarantee safe financial arithmetic and ensure output is formatted up to four decimal places.

## Generative AI Usage Declaration

I am declaring the use of Generative AI (Google Gemini) during the development of this exercise.

- I wrote the core logic, data structures, and the initial implementation of the state machine. I then used Gemini as a code reviewer (prompting it to "Analyze my code, check if everything makes sense according to the PDF, and suggest improvements").

- The AI helped me identify edge cases that I had initially overlooked.

- Also, the AI helped me to generate unit tests, and stress-cases to test the code.
