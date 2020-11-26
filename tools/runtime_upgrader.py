#!/bin/python3

from substrateinterface import SubstrateInterface, Keypair
from argparse import ArgumentParser
from loguru import logger


def load_payload(path):
    logger.info("Loading runtime ...")
    with open(path, "rb") as file:
        contents = file.read().hex()

    return '0x' + contents


def do_upgrade(secret_phrase, url, payload):
    logger.info("Doing runtime upgrade")
    custom_type_registry = {
            "runtime_id": 2,
            "types": {
                "NeuronMetadata": {
                    "type": "struct",
                    "type_mapping": [["ip", "u128"], ["port", "u16"], ["ip_type", "u8"]]
                }
            }
        }

    if secret_phrase[0:2] == "//":
        keypair = Keypair.create_from_uri(secret_phrase)
    else:
        keypair = Keypair.create_from_mnemonic(secret_phrase)

    substrate = SubstrateInterface(
        url=url,
        address_type=42,
        type_registry_preset='substrate-node-template',
        type_registry=custom_type_registry
    )

    call = substrate.compose_call(
        call_module='Sudo',
        call_function='sudo_unchecked_weight',
        call_params={
            '_weight': 0,
            'call': {
                'call_module': 'System',
                'call_function': 'set_code',
                'call_args': {
                    'code': payload
                }
            }
        }
    )

    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)

    try:
        result = substrate.submit_extrinsic(extrinsic)
        logger.info("Result: {}", result)
    except Exception as e:
        logger.error(e)
    finally:
        logger.info("Done")


if __name__ == '__main__':
    parser = ArgumentParser()
    parser.add_argument("--url", required=True, dest="url", help="The url of the node's RPC server")
    parser.add_argument("--runtime", required=True, dest="runtime",
                        help="The path to the runtime that needs to be uploaded."
                        "Usually located at: ./target/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.wasm")

    args = parser.parse_args()
    url = args.url
    runtime = args.runtime

    logger.info("Url {}", url)
    logger.info("Runtime: {}", runtime)
    secret_phrase = input("Secret phrase for sudo key: ")

    payload = load_payload(runtime)

    do_upgrade(secret_phrase, url, payload)
    secret_phrase = "X" * 500




