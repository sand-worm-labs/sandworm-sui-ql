# Sui Query Language (Sui\_Ql)

![image](https://github.com/user-attachments/assets/cfe1ba58-4da0-475e-a626-298946549451)

**Sui\_Ql** is a simple, SQL-like language for querying the Sui blockchain. It eliminates the complexity of interacting directly with the blockchain, handling RPC calls, rate limits, retries, and data formatting behind the scenes, so you can focus on writing your queries and getting results quickly.

---

## 🚀 Example Query

```sql
-- Fetch user’s balance and transactions from Sui
GET balance, transaction_id FROM account 0x123... ON sui
```

---

## 🧠 Why Sui\_Ql?

### The Problem

Querying Sui blockchain data traditionally requires:

* Writing complex and repetitive RPC calls
* Managing rate limits, retries, and different response formats
* Dealing with inconsistencies between various queries and data responses

### The Solution

**Sui\_Ql** simplifies everything:

* **SQL-like syntax** for simple, readable queries
* **Cross-chain querying** in one line, no need to switch between providers
* **Automatic data formatting** to standardize results
* **Zero boilerplate** — write the query, and you’re done

---

## ⚙️ How It Works

Sui\_Ql works in two main phases:

### 1. **Frontend Phase**

* Tokenizes and parses the query
* Validates the syntax
* Builds an Abstract Syntax Tree (AST) for query processing

### 2. **Backend Phase**

* Converts the AST into the appropriate Sui JSON-RPC calls
* Manages concurrent RPC requests
* Returns the results in consistent, structured output

For example, this query:

```sql
SELECT * FROM account 0xac5bceec1b789ff840d7d4e6ce4ce61c90d190a7f8c4f4ddf0bff6ee2413c33c, test.sui ON mainnet
```

Is transformed into:

* Resolving the account address `0x123...`
* Making necessary RPC calls (e.g., `getBalance`, `getTransactionCount`)
* Returning formatted, structured results

---

## ⚡ Quick Start

### Installation

```bash
# Install the Sui_Ql version manager
curl https://raw.githubusercontent.com/yourrepo/sui_ql/main/suiqlup/install.sh | sh

# Install the latest Sui_Ql version
suiqlup
```

---

### CLI Mode

```bash
suiql run query.sq    # Run a query file
suiql repl             # Start interactive REPL mode
```

---

### Library Mode

```toml
# Cargo.toml
[dependencies]
sui_ql_core = "0.1"
```

```rust
use sui_ql_core::interpreter::Interpreter;

#[tokio::main]
async fn main() {
    let result = Interpreter::run_program("GET balance FROM account 0x123... ON mainnet").await.unwrap();
    println!("{:?}", result);
}
```

---

### Web Mode

Try **Sui\_Ql** directly in the browser at [https://sui\_ql.sh](https://sui_ql.sh)

---

## 🔍 Supported Queries

### Entities

* `account`
* `checkpoint`
* `transaction`
* `object`

### Operations

* `GET`: Retrieve data
* `WHERE`: Apply filters to refine your query
* `ON`: Query across multiple chains in a single call
* Export: `CSV`, `JSON`, `Parquet` formats

---

## 📚 Documentation

* [Query Syntax Guide](#)
* [Installation Guide](#)

---

## 🛣 Roadmap

* ✅ Core Sui blockchain query support
* 🛠 Indexing layer (coming soon)
* 🔍 Advanced filtering and aggregation
* 🧪 Expanding test coverage
* 🌐 Support for additional blockchains

---

## 🤝 Contributing

We welcome contributions! Check out the [CONTRIBUTING.md](./CONTRIBUTING.md) for how to help out.

---

## 🪪 License

MIT License

---

This README should give you a clean and functional introduction to your **Sui\_Ql** project. Let me know if you need further tweaks!
