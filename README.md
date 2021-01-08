 ```commandline             _      _                                 
███████╗██╗   ██╗██████╗ ████████╗███████╗███╗   ██╗███████╗ ██████╗ ██████╗ 
██╔════╝██║   ██║██╔══██╗╚══██╔══╝██╔════╝████╗  ██║██╔════╝██╔═══██╗██╔══██╗
███████╗██║   ██║██████╔╝   ██║   █████╗  ██╔██╗ ██║███████╗██║   ██║██████╔╝
╚════██║██║   ██║██╔══██╗   ██║   ██╔══╝  ██║╚██╗██║╚════██║██║   ██║██╔══██╗
███████║╚██████╔╝██████╔╝   ██║   ███████╗██║ ╚████║███████║╚██████╔╝██║  ██║
╚══════╝ ╚═════╝ ╚═════╝    ╚═╝   ╚══════╝╚═╝  ╚═══╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝
                                                                             
```

                                                       
                                                                   





## Networks
We currently operate two different subtensor networks, each with their own chain.

### Kusanagi
Our main net is called Kusanagi. This is the live network.

### Akira
Akira is our staging network. This is the network which we will use to test new features, fixes, etc
before they are updated to the kusanagi main net. We urge you to use this network when you
are new to bittensor/subtensor and just want to try it out. 


## Installation

There are 2 ways to run subtensor
1) Run the binary directly
2) Install and run subtensor as a systemd unit



### Run the binary directly
If you run the binary directly, you will have some more options, such as connecting to different chains,
running your own chain, resetting the chain, printing debug information, etc.

#### Running on the akira test net
To run subtensor on the akira network, run the following command
```commandline
./bin/release/node-subtensor --chain akira
```

You should see output like this:
```commandline
Jan 08 10:23:36.268  INFO Subtensor Node
Jan 08 10:23:36.268  INFO ✌️  version 2.0.0-e76d087-x86_64-linux-gnu
Jan 08 10:23:36.268  INFO ❤️  by bittensor, 2020-2021
Jan 08 10:23:36.268  INFO 📋 Chain specification: Akira bittensor testnet
Jan 08 10:23:36.268  INFO 🏷  Node name: earsplitting-tiger-7228
Jan 08 10:23:36.268  INFO 👤 Role: FULL
Jan 08 10:23:36.268  INFO 💾 Database: RocksDb at /home/rawa/.local/share/node-subtensor/chains/akira_testnet/db
Jan 08 10:23:36.268  INFO ⛓  Native runtime: node-subtensor-runtime-1 (node-subtensor-client-1.tx1.au1)
Jan 08 10:23:36.371  WARN Using default protocol ID "sup" because none is configured in the chain specs
Jan 08 10:23:36.372  INFO 🏷  Local node identity is: 12D3KooWKpXvUKCVpHF3sXCSueZxu8fwXKPRxZMPZwj9BgUg5j2L (legacy representation: 12D3KooWKpXvUKCVpHF3sXCSueZxu8fwXKPRxZMPZwj9BgUg5j2L)
Jan 08 10:23:36.373  INFO 📦 Highest known block at #318
Jan 08 10:23:36.373  INFO 〽️ Prometheus server started at 127.0.0.1:9615
Jan 08 10:23:36.375  INFO Listening for new connections on 127.0.0.1:9944.
Jan 08 10:23:37.833  INFO 🔍 Discovered new external address for our node: /ip4/191.97.53.53/tcp/30333/p2p/12D3KooWKpXvUKCVpHF3sXCSueZxu8fwXKPRxZMPZwj9BgUg5j2L
Jan 08 10:23:41.376  INFO ⚙️  Syncing, target=#11024 (9 peers), best: #761 (0x9e4f…5b50), finalized #512 (0x6e4f…1709), ⬇ 176.5kiB/s ⬆ 6.3kiB/s
```

This line confirms you are running the subtensor node on the akira network:
```commandline
Jan 08 10:23:36.268  INFO 📋 Chain specification: Akira bittensor testnet
```

#### Running on the kusanagi main net
To run subtensor on kusanagi, run this command:
```commandline
./bin/release/node-subtensor
```

Take a look at this line to confirm you are running on kusanagi
```commandline
Jan 08 10:30:53.923  INFO 📋 Chain specification: Kusanagi bittensor main net
```

### Installation as a systemd unit
If you install subtensor as a systemd unit, this means it will connect to the kusanagi
main net. Do this when you are familiar with subtensor and bittensor and are ready to do
ML on these networks. 


To install subtensor, execute the install.sh script
```commandline
sudo ./install.sh
```

This should give the following output:
```shell script
[+] Copying ./bin/release/node-subtensor to /usr/local/bin/
[+] Creating user subtensor
[+] Creating data dir /var/lib/subtensor
[+] Setting ownership of /var/lib/subtensor to subtensor:subtensor
[+] Creating unit file /etc/systemd/system/subtensor.service
[+] Done!

--==[[ USEFUL COMMANDS ]]==--
Start subtensor : sudo systemctl start subtensor
Stop subtensor  : sudo systemctl stop subtensor
Start on reboot : sudo systemctl enable subtensor
Check status    : sudo systemctl status subtensor
```
