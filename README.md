# SubSocial Full Node by [DappForce](https://github.com/dappforce)

SubSocial is a set of Substrate runtime libraries (called FRAME) with UI that would allow anyone to launch their own decentralized censorship-resistant social network aka community. We are planning to follow a topology of Polkadot Network where every community will be running on its own Substrate chain and all these decentralized communities will be connected to our own Polkadot relay. This social networking relay could be connected to the official Polkadot Network.

You can think of this as decentralized versions of Reddit, Stack Exchange or Medium, where subreddits or communities of Stack Exchange or blogs on Medium run on their own chain. At the same time, users of these decentralized communities should be able to transfer or share their reputation, coins and other values from one community to another via Polkadot relay.

## Building from source

### Initial setup
If you want to build from source you will need the Rust [toolchain](https://rustup.rs/), openssl and llvm/libclang:
```bash
curl https://sh.rustup.rs -sSf | sh
sudo apt install make clang pkg-config libssl-dev
rustup update
```
Then, you should have cloned Subsocial Substrate repository:
```bash
git clone https://github.com/dappforce/dappforce-subsocial-node
```

### Building
Initialise the WASM build environment:
```bash
cd dappforce-subsocial-node/
./scripts/init.sh
```

Build the node (native code):
```bash
cargo build --release
```

### Running a public node
[!] Experimental and may not work. Better run [with Docker](#connect-to-our-network-with-docker)
Run the node and connect to the public Subsocial Testnet.
```bash
./target/release/subsocial-node --name "Node Name" --pruning archive --bootnodes /ip4/167.172.104.62/tcp/30333/p2p/QmfHoYER3gPchRvHSTzyFZ9hhvQG3oP9jgcQEbfbm2x9LR /ip4/167.172.104.62/tcp/30334/p2p/QmVcp8pBKtKcwWmWcrARZcdLxW9E1EzfU9JwvgiSKu4How
```

### Installing a release build
This will install the executable `subsocial-node` to your `~/.cargo/bin` folder, which you would normally have in your `$PATH` environment.

```bash
cargo install --path ./
```

Now you can run

```bash
subsocial-node
```

## Building from Docker

### Easiest start
To start local Subsocial Node (you should have docker-compose):

```
cd docker/
docker-compose up -d
```

### Start with your own  parameters

```
docker run -p 9944:9944 dappforce/subsocial-node:latest ./subsocial-node [flags] [options]
```
* Don't forget `--ws-external` flag, if you want your node to be visible no only within the container.

### Connect to our network with Docker
If you're interested in launching your Node that will be connected to Subsocial Testnet, you should run the next command:
```
docker run -p 30333:30333 -p 9933:9933 -v <YourPath>:/data dappforce/subsocial-node:df-v2 ./subsocial-node -d /data --name <NodeName> --pruning archive --bootnodes /ip4/167.172.104.62/tcp/30333/p2p/QmfHoYER3gPchRvHSTzyFZ9hhvQG3oP9jgcQEbfbm2x9LR /ip4/167.172.104.62/tcp/30334/p2p/QmVcp8pBKtKcwWmWcrARZcdLxW9E1EzfU9JwvgiSKu4How
```
, where:
`<YourPath>` - should be replaced with a full path, where node's data will be stored;
`<NodeName>` - node's name you wish to be shown on a [telemetry](https://telemetry.polkadot.io/#list/Subsocial%20Barracuda%20Testnet) webpage.
Be patient, current active bootnodes' URIs are:
- Alice: `/ip4/167.172.104.62/tcp/30333/p2p/QmfHoYER3gPchRvHSTzyFZ9hhvQG3oP9jgcQEbfbm2x9LR`
- Bob: `/ip4/167.172.104.62/tcp/30334/p2p/QmVcp8pBKtKcwWmWcrARZcdLxW9E1EzfU9JwvgiSKu4How`

### Build your own image
If you want to build docker image from your local repository (it takes a while...), run the following command from the root of the repository:

```
docker build -f docker/Dockerfile <YourName>/subsocial-node:latest .
```
, where:
- `<YourName>` - your username on hub.docker.com or any other you wish.

### Start all parts of Subsocial at once with [Subsocial-Starter](https://github.com/dappforce/dappforce-subsocial-starter)

## Development

### Running a local development node
```bash
./target/release/subsocial-node --dev
```

### Cleaning development chain data
When making changes to the runtime library remember to purge the chain after rebuilding the node to test the new runtime.

```bash
./target/release/subsocial-node purge-chain --dev
```
