#!/bin/python3

import argparse
import subprocess
import json, yaml
from loguru import logger
from pathlib import Path


tmp_path_str = '/tmp/pychainspec/'
json_path_str = tmp_path_str + 'dumped_chain_spec.json'
new_json_path_str = tmp_path_str + 'new_chain_spec.json'


'''
Step 0: Make tmp dir to dump json into
Step 1: Export chainspec json from node binary
Step 2: Load chainspec json from file system into object
Step 3: Load chain spec settings from yaml file
Step 4: Fill chainspec object with values from json yaml file
Step 5: Save new chain spec as json file
Step 6: Convert chainspec json into raw chain spec
'''


parser = argparse.ArgumentParser(description="Dumps a chainspec from a node binary, overlays values from config yaml, "
                                             "converts into raw chainspec json. Values that are encountered in the "
                                             "config are used to overwrite the corresponding values in the json. "
                                             "If the value exists in the config but not in the json, it is added to the"
                                             " json. If the value is a list, the --lists parameter specifies what to "
                                             "do. The default behavior is 'append'")
parser.add_argument("--binary", required=True, help="Path to the node binary")
parser.add_argument("--config", required=True, help="Path to the chain spec configuration yaml")
parser.add_argument("--dest", required=True, help="Path to the raw chain spec json file")
parser.add_argument("--lists", choices=['overwrite', 'append'], default='append',
                    help="Specifies the behavior when a list is encountered. 'overwrite' will overwrite existing "
                         "values, 'append' will append the config yaml's values to the existing values")
args = parser.parse_args()


bin_path = args.binary
cfg_path = args.config
dest_path = args.dest
lists_behaviour = args.lists

# Step 0
logger.info("Creating path {} if it does not exist already", tmp_path_str)
tmp_path = Path(tmp_path_str)
tmp_path.mkdir(exist_ok=True)

#Step 1
logger.info("Dumping chainspec from {} to {}", bin_path, json_path_str)

try:
    with open (json_path_str, 'w') as outfile:
        subprocess.run([bin_path, 'build-spec', '--disable-default-bootnode'], stdout=outfile)
        outfile.close()
except FileNotFoundError:
    logger.error("File {} not found, exiting", json_path_str)
    quit(-1)

#Step 2
logger.info("Applying config to chain spec")
try:
    with open(json_path_str, 'r') as infile:
        json_obj = json.load(infile)
        infile.close()
except FileNotFoundError:
    logger.error("File {} not found. Exiting", json_path_str)
    quit(-1)

#Step 3 - Load chain spec config from yaml
try:
    with open(cfg_path, 'r') as infile:
        yaml_obj = yaml.safe_load(infile)
        infile.close()
except FileNotFoundError:
    logger.error("File {} not found, exiting.", cfg_path)
    quit(-1)


#Step 4 - Fill json with yaml values

def traverse(a, b):
    for key, val in a.items():
        if key not in b:
            logger.warning("Key '{}' does not exist in json. Key is added, but you might want to review this.", key)
            b[key] = val

        if isinstance(val, dict):
            traverse(a[key], b[key])
        elif lists_behaviour == 'append' and isinstance(val, list):
            b[key] += val
        else:
            b[key] = val


traverse(yaml_obj, json_obj)

# Step 5 - Save to temp file
logger.info("Saving new json to {}", new_json_path_str)
try:
    with open(new_json_path_str, 'w') as outfile:
        json.dump(json_obj, outfile)
        outfile.close()

except PermissionError:
    logger.error("A permission error occured while writing to {}. Exiting.", new_json_path_str)


# Step 6 - Convert to raw file that can be read by a node.

logger.info("Converting {} into raw format, storing the result in {}. ", new_json_path_str, dest_path)
try:
    with open(dest_path, 'w') as outfile:
        subprocess.run([bin_path, 'build-spec', '--chain=%s' % new_json_path_str,
                        '--raw', '--disable-default-bootnode'], stdout=outfile)
        outfile.close()
except PermissionError:
    logger.error("A permission error occured while writing to {}", dest_path)


# Print some useful information on how to use the new chain

logger.info("Done")
logger.info("If would like to see the result of the operation in human readable JSON, open {}", new_json_path_str)
logger.info("You can run the node with the new chain spec as follow:")
logger.info("{} --chain={}", bin_path, dest_path)






