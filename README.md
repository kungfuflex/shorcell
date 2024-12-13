# shorcell

Post-quantum container for a virtualized UTXO, built on ALKANES.

## Background

Balance sheets of alkanes assets as they exist on the metaprotocol are tracked on individual UTXOs, but also a virtualized balance sheet is tracked on alkanes themselves. We use the term `orbital` to describe a unary alkane, or otherwise an alkane that is, in a sense, non-fungible. It is simply an alkane with a max total supply of 1.

Now consider a world where ere we do not consider any normal spendable UTXO to be safe to store value on, because as we all know, we are approaching the **POST-QUANTUM AI GIRLFRIEND ROBOT OVERLORD FUTURE 🔒🔒🔒**. Thus, this repository contains sources for an orbital which represents a virtual UTXO, as we normally understand the concept, that can only be spent with a witness envelope reveal of a SPHINCS+-128s signature, offering us strong security and safety against shor's algorithm and the looming threat of quantum apocalypse.

Since Bitcoin consensus itself is secured by sha256, which is quantum-safe, the `shorcell` concept and the underlying mechanics of protorunes gives us a way to store value, in the form of metaprotocol assets, on Bitcoin, without eventually being compromised by a single malicious actor with sufficient qubits to break ECDSA and spend vulnerable outputs.

There is still hope, apparently.

## Build

```sh
cargo build --release
```

WASM will be built to `target/wasm32-unknown-unknown/release/shorcell.wasm`

gzip compression level 9 is recommended to compress the wasm to a `*.wasm.gz` file before deploying to Bitcoin.

## Usage

A shorcell is instantiated via a factory operation along with some inputs of alkanes and a vout index targeting a p2tr output, which should simply contain some dust. The p2tr output is not meant to be spendable, and if it were, it would have no effect. It simply has the correct bytesize for a UTXO to encode a 32 byte public key associated with normal usage of SPHINCS+-128s.

The shorcell alkane will accept the balance sheet of alkanes input to the transction and hold it within its runtime, then associate the 32-byte public key stored on the p2tr output for the transaction with the owner of the balance sheet that lives on the alkane. The alkane can, at any time, be spent by any transaction that includes a witness envelope containing a SPHINCS+-128s signature of the transaction bytes with the witness stack payload omitted from the message digest.

The protomessage associated with a shorcell spend sets the pointer to the output where the balance sheet should be transferred to. This can be a normal quantum-vulnerable output the same as the ones we will see get pwned in the future, or you can simply target another shorcell initialization, where the p2tr output created can be a quantum address of your desired recipient. This way we can compose value stored in shorcells into normal transactions on alkanes, but also have a mechanism to do simple transfers to other entities to their quantum address, which is equivalent in structure to a conventional p2tr address.

## Opcodes 

This alkane implements the following opcodes:

- 0: `initialize(u128: p2tr_output_vout)
- 78: `burn()`
- 99: `name(): String`
- 100: `symbol(): String`


## Author

flex

## License

MIT
