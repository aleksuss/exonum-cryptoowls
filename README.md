# Exonum cryptoowls

Demo created with [Exonum blockchain](https://github.com/exonum/exonum).

![Exonum cryptoowl](cryptoowl.png)

## Demo summary

* Register a new user and get an initial owls;
* Breed your owls and get a new unique owls;
* Sell your owls using auction model;
* Buy owls from other users.

## Getting started

Demo is supported by MacOS, Linux and Windows platforms.

Check prerequisites for your operating system
at [Exonum Documentation](https://exonum.com/doc/get-started/install/).

Then install following necessary packages:

* [git](https://git-scm.com/downloads)
* [Rust](https://rustup.rs/)
* [Node.js & npm](https://nodejs.org/en/download/)

## Build and run

Below you will find a step-by-step guide to start the service
on 4 nodes on the local machine.

Clone the project and install Rust dependencies:

```sh
git clone https://github.com/exonum/exonum-cryptoowls

cd exonum-cryptoowls

cargo install
```

Generate blockchain configuration:

```sh
mkdir example

exonum-cryptoowls generate-template example/common.toml --validators-count 4
```

Generate templates of nodes configurations:

<!-- markdownlint-disable MD013 -->

```sh
exonum-cryptoowls generate-config example/common.toml  example/pub_1.toml example/sec_1.toml --peer-address 127.0.0.1:6331

exonum-cryptoowls generate-config example/common.toml  example/pub_2.toml example/sec_2.toml --peer-address 127.0.0.1:6332

exonum-cryptoowls generate-config example/common.toml  example/pub_3.toml example/sec_3.toml --peer-address 127.0.0.1:6333

exonum-cryptoowls generate-config example/common.toml  example/pub_4.toml example/sec_4.toml --peer-address 127.0.0.1:6334
```

Finalize generation of nodes configurations:

```sh
exonum-cryptoowls finalize --public-api-address 0.0.0.0:8200 --private-api-address 0.0.0.0:8091 example/sec_1.toml example/node_1_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml

exonum-cryptoowls finalize --public-api-address 0.0.0.0:8201 --private-api-address 0.0.0.0:8092 example/sec_2.toml example/node_2_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml

exonum-cryptoowls finalize --public-api-address 0.0.0.0:8202 --private-api-address 0.0.0.0:8093 example/sec_3.toml example/node_3_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml

exonum-cryptoowls finalize --public-api-address 0.0.0.0:8203 --private-api-address 0.0.0.0:8094 example/sec_4.toml example/node_4_cfg.toml --public-configs example/pub_1.toml example/pub_2.toml example/pub_3.toml example/pub_4.toml
```

Run nodes:

```sh
exonum-cryptoowls run --node-config example/node_1_cfg.toml --db-path example/db1 --public-api-address 0.0.0.0:8200

exonum-cryptoowls run --node-config example/node_2_cfg.toml --db-path example/db2 --public-api-address 0.0.0.0:8201

exonum-cryptoowls run --node-config example/node_3_cfg.toml --db-path example/db3 --public-api-address 0.0.0.0:8202

exonum-cryptoowls run --node-config example/node_4_cfg.toml --db-path example/db4 --public-api-address 0.0.0.0:8203
```

<!-- markdownlint-enable MD013 -->

Install Node.js dependencies:

```sh
cd frontend

npm install
```

Build proto files:

```sh
npm run proto
```

Build application:

```sh
npm run build
```

Run application:

```sh
npm start -- --port=3000 --api-root=http://127.0.0.1:8200
```

`--port` Node.js application port.

`--api-root` root URL of node's public API.

Ready! Application can be reached
at [http://127.0.0.1:3000](http://127.0.0.1:3000).

## License

Cryptoowls demo is licensed under the Apache License (Version 2.0).
See [LICENSE](LICENSE) for details.
