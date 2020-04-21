# Subsocial Full Node by [DappForce](https://github.com/dappforce)

Subsocial is a set of Substrate pallets with web UI that allows anyone to launch their own decentralized censorship-resistant social network aka community. Every community can be a separate Substrate chain and connect with other communities via a Polkadot-based relay chain.

You can think of this as decentralized versions of Reddit, Stack Exchange or Medium, where subreddits or communities of Stack Exchange or blogs on Medium run on their own chain. At the same time, users of these decentralized communities should be able to share their reputation or transfer coins and other values from one community to another via Polkadot relay chain.

To learn more about Subsocial, please visit [Subsocial Network](http://subsocial.network).

## Supported by Web3 Foundation

<img src="https://github.com/dappforce/dappforce-subsocial/blob/master/w3f-badge.svg" width="100%" height="200" alt="Web3 Foundation grants badge" />

Subsocial is a recipient of the technical grant from Web3 Foundation. We have successfully delivered all three milestones described in Subsocial's grant application. [Official announcement](https://medium.com/web3foundation/web3-foundation-grants-wave-3-recipients-6426e77f1230).

## Building from source

### Requirements
If you want to build Subsocial node from source, you need the next tools installed: Rust [toolchain](https://rustup.rs/), openssl and llvm/libclang:
```bash
curl https://sh.rustup.rs -sSf | sh
sudo apt install make clang pkg-config libssl-dev
rustup update
```
Then clone this repository:
```bash
git clone https://github.com/dappforce/dappforce-subsocial-node
```

### Build
Initialise the WASM build environment:
```bash
cd dappforce-subsocial-node/
./scripts/init.sh
```

Build the node (native code):
```bash
cargo build --release
```

### Run as a public node
[!] Experimental and may not work. Better [run with Docker](#connect-to-our-network-with-docker)
Run as an archive node and connect to the public [Subsocial Testnet](http://testnet.subsocial.network/).
```bash
./target/release/subsocial-node --name "Node Name" --pruning archive --bootnodes /ip4/167.172.104.62/tcp/30333/p2p/QmfHoYER3gPchRvHSTzyFZ9hhvQG3oP9jgcQEbfbm2x9LR /ip4/167.172.104.62/tcp/30334/p2p/QmVcp8pBKtKcwWmWcrARZcdLxW9E1EzfU9JwvgiSKu4How
```

### Install a release build
This will install an executable `subsocial-node` to your `~/.cargo/bin` folder, which you would normally have in your `$PATH` environment.

```bash
cargo install --path ./
```

Now you are ready to run the node:

```bash
subsocial-node
```

## Build with Docker

### Easiest way
To start a local Subsocial node (you should have `docker-compose` installed):

```
cd docker/
docker-compose up -d
```

### Start with custom parameters

```
docker run -p 9944:9944 dappforce/subsocial-node:latest ./subsocial-node [flags] [options]
```
* Don't forget `--ws-external` flag, if you want your node to be visible outside of the docker container.

### Connect to our network with Docker
If you're interested in launching your node and connect to [Subsocial Testnet](http://testnet.subsocial.network/), you should run the next command:
```
docker run -p 30333:30333 -p 9933:9933 -v <YourPath>:/data dappforce/subsocial-node:df-v2 ./subsocial-node -d /data --name <NodeName> --pruning archive --bootnodes /ip4/167.172.104.62/tcp/30333/p2p/QmfHoYER3gPchRvHSTzyFZ9hhvQG3oP9jgcQEbfbm2x9LR /ip4/167.172.104.62/tcp/30334/p2p/QmVcp8pBKtKcwWmWcrARZcdLxW9E1EzfU9JwvgiSKu4How
```
where:
`<YourPath>` - should be replaced with a full path, where node's data will be stored;
`<NodeName>` - node's name you wish to be shown on [Telemetry](https://telemetry.polkadot.io/#list/Subsocial%20Barracuda%20Testnet).
Be patient, the current active bootnodes' URIs are:
- Alice: `/ip4/167.172.104.62/tcp/30333/p2p/QmfHoYER3gPchRvHSTzyFZ9hhvQG3oP9jgcQEbfbm2x9LR`
- Bob: `/ip4/167.172.104.62/tcp/30334/p2p/QmVcp8pBKtKcwWmWcrARZcdLxW9E1EzfU9JwvgiSKu4How`

### Build your own Docker image
If you want to build a docker image from your local repository (it will take some time...), run the following command from the root of your repository:

```
docker build -f docker/Dockerfile <YourName>/subsocial-node:latest .
```
, where:
- `<YourName>` - your username on [hub.docker.com](https://hub.docker.com) or any other username of your choice.

### Start all parts of Subsocial at once with [Subsocial Starter](https://github.com/dappforce/dappforce-subsocial-starter)

## Development

### Run a local node in development mode
```bash
./target/release/subsocial-node --dev
```

### Clean a development chain data
When making changes to the runtime library remember to purge the chain after rebuilding the node to test the new runtime.

```bash
./target/release/subsocial-node purge-chain --dev
```
