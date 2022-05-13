# Join KICO collators

`KICO` welcomes community members to join `KICO` collator. The following method is our recommended method

## Requirements

For the full details of the requirements:

* `System`: Ubuntu 20.04
* `CPU`: >4CPU
* `Memory`: >8G
* `Storage`: >300G

## Register as candidate

You first have to prepare some available KICOs

The steps are as following:

0. Run KICO node

[KICO node](https://github.com/DICO-TEAM/spec-setup)

1. Launch collator and insert key

```
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "author_insertKey",
    "params": ["<aura/gran>", "<mnemonic phrase>", "<public key>"]
}'
```

2. Rotate keys

```
Developer -> RPC calls -> author -> rotateKeys() -> Submit RPC call
```

3. Set session keys

```
Developer -> Extrinsics -> session -> setKeys(keys, proof)
```

4. Register as collator candidate

```
Developer -> Extrinsics -> collatorSelection -> registerAsCandidate()
```

5. Wait the next session (about 6 * HOURS)

[collators](https://polkadot.js.org/apps/#/collators) will contain your node.