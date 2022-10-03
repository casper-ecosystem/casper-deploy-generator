# Casper Deploy Generator

Test vectors for Ledger hardware integration with CasperNetwork.

## Background

The input(s) to the process is a collection of `Deploy` samples, each representing a (slightly) different transaction a user could issue. Some samples may be valid, others are invalid - it is indicated by the validity flags and by `valid_regular`/`valid_expert` fields in the final `manual.json` file. The file itself contains a data samples structured as per Zondax's requirements.

Ledger represents a transaction as a series of "pages" - each page presenting a fraction of the transaction, limited by the Ledger's physical constraints (number of characters per line, number of lines to display the page). In the process of confirming a transaction, user has to click through all of the pages, reading and validating each, and in the end either approve it or reject it. Not all elements of a transaction can fit into a single Ledger page, if that's the case then that element spans multiple pages and Ledger displays `[n/m]` as part of the label. Example:
```
// first page in Ledger hardware
Account [1/2] 
0202531Fe60681345
03D2723133227c867
// second page in Ledger hardware
Account [2/2] 
Ac8Fa6C83C537e9a4
4c3c5BdBDCb1fE337
```

## Goals & requirements
Goal of this project is to provide Zondax with a set of test vectors that will cover all possible CasperLabs' transaction variants and reference representations as for how to display them in the Ledger hardware.

Currently, we support the following transaction types:
* Native token (CSPR) transfer.
* Auction actions: delegate, undelegate, redelegate.
* Generic transactions.

Each representation should be sufficient and succint: 
* sufficient - user needs to verify all the important parts of the transaction and be sure that it indeed represents a transaction he/she is submitting.
* succint - user shouldn't be required to click through dozens of "pages" as that may lead to cognitive overload and approving the txn without validating of its parts.

## Ledger representations for various transaction types

For every transacation type there is a set of fields that are always present, regardless of what the rest of the transaction is. These fields are:
* **Txn hash** - short blake2b hash of the whole transaction. Can be used to cross-check the whole transaction with a web wallet that presents more data with additional details
* **Type** - high-level type of the transaction. Currently, we support following types: _delegate, undelegate, redelegate, token transfer, contract execution_
* **Chain ID** - human-readable ID of the chain for which the transaction is aimed at. This field is verified by the receiving node and in the case of mismatch between _chain ID_ from the transaction and that of the receiving network rejects the transaction.
* **Account** - public key (with a signing algorithm tag prepended - 01 or 02) of the account creating the transaction.
* **Fee** - fee for the transaction.

For the sake of brevity, these fields will be omitted in the specific description below. Reader can assume they are always present.

Additionally, each transaction includes **Execution** field (visible only in expert mode) specifying type of the call the transaction is making:
* `by-hash` - address of the contract this txn is calling
* `by-hash-versioned` - address of the contract txns is calling and its version
* `by-name` - name of the contract (as stored in _named keys_ of the `Account) this txn is calling
* `by-name-versioned` - name of the contract (as stored in _named keys_ of the `Account) this txn is calling AND its version

### Note on the _expert mode_

Ledger apps allow user to choose between _regular_ and _expert_ modes for displaying transaction information. There is no definitive guidelines about which fields should be _expert-only_ and which not so the choice is subjective but our rule was that if a piece of information may lead to user being tricked into signing an unexpected transaction, then that field should be present in _regular_ mode.

The following fields are displayed only in _expert_ mode:
* **Timestamp** - timestamp of transaction creation
* **Ttl** - time-to-live of the transaction
* **Deps #** - number of transaction dependencies
* **ID** - (native transfer only and optional, defaults to 0) ID of the native tranfser
* **Approvals #** - number of keys that have signed the transaction so far

### Native token transfer
Transfer of native (CSPR) tokens between two accounts (or purses). We choose to display:
* **Target** - recipient of the transfer
* **Amount** - amount of CSPRs (in motes) being transferred

### Delegate
An action of delegating tokens to a validator to participate in staking rewards:
* **Delegator** - source of the tokens for delegation
* **Validator** - address of the validator we're delegating to
* **Amount** - amount of tokens being delegated

### Undelegate
An action of removing delegated tokens. After that, the delegator will stop receiving staking rewards:
* **Delegator** - source of the tokens for undelegation
* **Validator** - address of the validator we're undelegating from
* **Amount** - amount of tokens being delegated

### Redelegate
An action of switching validators we're delegating to. Different from _undelegate + delegate_ as it's not subject to additional bonding period. 
* **Delegator** - source of the tokens for redelegation
* **Old** - address of the old validator we're undelegating from
* **New** - address of the old validator we're delegating to
* **Amount** - amount of tokens we're moving between validators

NOTE: Unfortunately, _old validator_ and _new validator_ labels would exceed the 11 char limit of the Ledger hardware.

### Generic transaction
Any transaction that isn't any of the above. CasperNetwork transaction structure is very flexible but b/c of it it's also very difficult to parse (for example argument to a contract call can be infinitely recursive structure - `Vec<Vec<Vec<...>>>`) in an environment as limited as Ledger (limited stack memory).

For cases like that, Ethereum Ledger app has a notion of _blind signing_ but that isn't very secure as Ledger presentation hides almost all information about the transaction from the user, basically preventing it from validating. See an article about it from CTO of Ledger: https://cointelegraph.com/news/ledger-cto-warns-crypto-users-about-the-dangers-of-blind-signing.

Given all that - complexity of CasperNetwork transactions and desire to not hide important details from users - we chose to present the following parts for _generic transactions_ (apart from previously listed elements in [the introduction to this section](#ledger-representations-for-various-transaction-types)):
* **Execution** - type of contract call (by name/hash; specific version/latest version)
* **Name**/**Address** - name of address of the contract being called
* **Version** - latest or specific
* **Args hash** - blake2b hash for serialized arguments of the transaction.

The last point deserves more explanation. As stated earlier, complexity of CasperNetwork transaction comes mostly from its arguments but it's also the arguments that influence how it affects the state - am I transferring tokens to someone I trust or not? Am I calling this swap with a slippage I accepted? etc. We chose to display the hash of the arguments as a succint representation of it, knowing that even the slightest modification to any of the arguments will affect the resulting hash. CasperNetwork Ledger app is called from a web wallet (cspr.live), other dApps or browser extension, we rely on those (and hope) to present user with all the relevant arguments of transaction AND their hash, allowing the Ledger user to cross-check the **Args hash** from the Ledger app with the one in the wallet/extension.
## Code structure

The core element of the code is a generic [`Sample<T>`](./src/sample.rs) structure, for our purposes we can assume it's `Sample<Deploy>`. It represents a sample, singular test vector (single transaction) for the pipeline. 

Given sample `Deploy` instance, we first parse it to [`Ledger`](./src/ledger.rs#L85) structure that maps `Deploy` to a series of transaction [`Element`](./src/ledger.rs#L40)s - each with its own label, value and `expert` flag. At this point, `Element`'s value isn't yet "chopped up" to span multiple Ledger hardware pages. That's what [`LedgerPageView::from_element`](./src/ledger.rs#L159) is for - it maps individual `Element`s into proper "Ledger pages".

This architecture may seem unnecessarily complicated but it separates cleanly Ledger mechanics from CasperNetwork specific types. One would need to implement a different parser, turning transaction into `Vec<Element>` and plug into the rest of the flow, to build a new Zondax-compliant Ledger test vector generator.

If you dig into the code deeper, you may find [`LimitedLedgerView`](./src/ledger.rs#L278) struct. It's a wrapper around `Ledger` instance and `LimitedLedgerConfig`. Its purpose is to trigger additional handling logic that if _regular_ (or _expert_) representation of the transaction matches the criteria. For example, if _regular_ mode presentation contained too many pages, Ledger app could choose to display an INFO message asking user to switch to _expert_ before approving.

## Data schema

`manual.json` file contains test vectors in the format that is expected by the Zondax tools. It is a collection of individual test vector with the following schema (example):
```json
{
    "index": 0,
    "name": "undelegate__type_by_hash__payment_system",
    "valid_regular": true,
    "valid_expert": true,
    "testnet": true,
    "blob": "<<redacted for readability. contains serialized representation of the transaction>>",
    "output": [
      "0 | Txn hash [1/2] : 871193cE8e7392578c4455f350Decf9a1a",
      "0 | Txn hash [2/2] : 55d63ee6e62Bce367c12799d344D58",
      "1 | Type : Undelegate",
      "2 | Chain ID : mainnet",
      "3 | Account [1/2] : 0202531Fe6068134503D2723133227c867",
      "3 | Account [2/2] : Ac8Fa6C83C537e9a44c3c5BdBDCb1fE337",
      "4 | Fee : 1 000 000 000 motes",
      "5 | Delegator [1/2] : 0101010101010101010101010101010101",
      "5 | Delegator [2/2] : 01010101010101010101010101010101",
      "6 | Validator [1/2] : 0103030303030303030303030303030303",
      "6 | Validator [2/2] : 03030303030303030303030303030303",
      "7 | Amount : 0 motes"
    ],
    "output_expert": [
      "0 | Txn hash [1/2] : 871193cE8e7392578c4455f350Decf9a1a",
      "0 | Txn hash [2/2] : 55d63ee6e62Bce367c12799d344D58",
      "1 | Type : Undelegate",
      "2 | Chain ID : mainnet",
      "3 | Account [1/2] : 0202531Fe6068134503D2723133227c867",
      "3 | Account [2/2] : Ac8Fa6C83C537e9a44c3c5BdBDCb1fE337",
      "4 | Timestamp : 2021-05-04T14:20:35Z",
      "5 | Ttl : 1day",
      "6 | Gas price : 2",
      "7 | Deps # : 3",
      "8 | Fee : 1 000 000 000 motes",
      "9 | Execution : by-hash",
      "10 | Address [1/2] : 0101010101010101010101010101010101",
      "10 | Address [2/2] : 010101010101010101010101010101",
      "11 | Delegator [1/2] : 0101010101010101010101010101010101",
      "11 | Delegator [2/2] : 01010101010101010101010101010101",
      "12 | Validator [1/2] : 0103030303030303030303030303030303",
      "12 | Validator [2/2] : 03030303030303030303030303030303",
      "13 | Amount : 0 motes",
      "14 | Approvals # : 10"
    ]
  }
```


## How to run

In order to generate test vectors, run:

```bash
make test-vectors
```

Output of the execution is included in `manual.json` file.
