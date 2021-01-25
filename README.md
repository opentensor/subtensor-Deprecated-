 ```commandline             _      _                                 
███████╗██╗   ██╗██████╗ ████████╗███████╗███╗   ██╗███████╗ ██████╗ ██████╗ 
██╔════╝██║   ██║██╔══██╗╚══██╔══╝██╔════╝████╗  ██║██╔════╝██╔═══██╗██╔══██╗
███████╗██║   ██║██████╔╝   ██║   █████╗  ██╔██╗ ██║███████╗██║   ██║██████╔╝
╚════██║██║   ██║██╔══██╗   ██║   ██╔══╝  ██║╚██╗██║╚════██║██║   ██║██╔══██╗
███████║╚██████╔╝██████╔╝   ██║   ███████╗██║ ╚████║███████║╚██████╔╝██║  ██║
╚══════╝ ╚═════╝ ╚═════╝    ╚═╝   ╚══════╝╚═╝  ╚═══╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝
                                                                             
```

## System Requirements
* The binaries in ./bin/release are x86_64 binaries to be used with the Linux kernel.  
* Subtensor needs ~286 MiB to run.                      
* Architectures other than x86_64 are currently not supported.
* OSs other than Linux are currently not supported.               

## Network requirements
* Subtensor needs access to the public internet
* Subtensor runs on ipv4
* Subtensor listens on the following ports:
1) 9944 - Websocket. This port is used by bittensor. It only accepts connections from localhost. Make sure this port is firewalled off from the public domain.
2) 9933 - RPC. This port is opened, but not used.
3) 30333 - p2p socket. This port accepts connections from other subtensor nodes. Make sure your firewall(s) allow incoming traffic to this port.

* It is assumed your default outgoing traffic policy is ACCEPT. If not, make sure outbound traffic to port 30333 is allowed.



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
First of all, the genesis block of the akira network needs to be installed. This step needs to be repeated
if you every purge the akira chain.   
  
Warning!!! Only do this when you are installing a fresh copy of subtensor or when you have purged the chain.
This operation will reset the chain to 0, so your node will have to sync the chain to its current height.
So do not do this when you are installing an update.
```
mkdir -p ~/.local/share/node-subtensor
tar -xf ./bin/release/akira_genesis.tar -C ~/.local/share/node-subtensor
```




Then, to run subtensor on the akira network, run the following command
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
First of all, the genesis block of the kusanagi network needs to be installed. This step needs to be repeated
if you every purge the kusanagi chain.   
   
Warning!!! Only do this when you are installing a fresh copy of subtensor or when you have purged the chain.
This operation will reset the chain to 0, so your node will have to sync the chain to its current height.
So do not do this when you are installing an update.
```
mkdir -p ~/.local/share/node-subtensor
tar -xf ./bin/release/kusanagi_genesis.tar -C ~/.local/share/node-subtensor
```

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
[+] Creating data dir /var/lib/subtensor                                        
[+] Checking if kusanagi chain data is already present
[+] kusanagi chain data is not present. This indicates a fresh install. Installing genesis block
[+] Setting ownership of /var/lib/subtensor and subdirs to subtensor:subtensor
[+] Creating unit file /etc/systemd/system/subtensor.service
[+] Done!


--==[[ USEFUL COMMANDS ]]==--
Start subtensor : sudo systemctl start subtensor
Stop subtensor  : sudo systemctl stop subtensor
Start on reboot : sudo systemctl enable subtensor
Check status    : sudo systemctl status subtensor
```


You can start the subtensor node using the following command:
```commandline
sudo systemctl start subtensor
```

If you use this command:
```commandline
sudo systemctl status subtensor
```

You should see an output as this:
```commandline
     Loaded: loaded (/etc/systemd/system/subtensor.service; disabled; vendor preset: disabled)
     Active: active (running) since Mon 2021-01-25 17:04:39 -05; 2s ago
   Main PID: 61795 (node-subtensor)
      Tasks: 40 (limit: 9251)
     Memory: 30.5M
     CGroup: /system.slice/subtensor.service
             └─61795 /usr/local/bin/node-subtensor --base-path /var/lib/subtensor

Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.554  INFO 📋 Chain specification: Kusanagi bittensor main net
Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.554  INFO 🏷  Node name: wicked-zoo-7246
Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.554  INFO 👤 Role: FULL
Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.554  INFO 💾 Database: RocksDb at /var/lib/subtensor/chains/kusanag>
Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.555  INFO ⛓  Native runtime: node-subtensor-runtime-4 (node-subtens>
Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.955  WARN Using default protocol ID "sup" because none is configure>
Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.955  INFO 🏷  Local node identity is: 12D3KooWHafxz73rjJgiQXaMmWttiV>
Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.957  INFO 📦 Highest known block at #17668
Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.958  INFO 〽️ Prometheus server started at 127.0.0.1:9615
Jan 25 17:04:39 bittensor node-subtensor[61795]: Jan 25 17:04:39.960  INFO Listening for new connections on 127.0.0.1:9944.
```


## Troubleshooting
If you ever run into this problem:
```commandline
Jan 25 16:20:42.317  INFO 🔍 Discovered new external address for our node: /ip4/191.97.53.53/tcp/30333/p2p/12D3KooWKpXvUKCVpHF3sXCSueZxu8fwXKPRxZMPZwj9BgUg5j2L
Jan 25 16:20:45.402 ERROR Bootnode with peer id `12D3KooWEr7Dq9oFJRSXZrZspibBLRySnGCDV7598xrGF8iT5DHD` is on a different chain (our genesis: 0x8db6…31bf theirs: 0xaca1…8c79)
Jan 25 16:20:45.616 ERROR Bootnode with peer id `12D3KooWAcwbhijTx8NB5P9sLGcWyf4QrhScZrqkqWsh418Nuczd` is on a different chain (our genesis: 0x8db6…31bf theirs: 0xaca1…8c79)
```

It means that the genesis block for the network you're trying to access is not correctly set-up.
Most likely, you have purged the chain, so you will need to reinit the genesis block.
Follow the step under installation pertaining to the genesis block to do this