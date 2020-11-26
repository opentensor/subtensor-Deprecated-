#!/bin/python3

"""
MIT License

Copyright (c) 2020 opentensor

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""

from substrateinterface import SubstrateInterface, Keypair
from argparse import ArgumentParser
from loguru import logger
from pathlib import Path
import validators


def read_file(path : Path) -> str:
    """
    Loads the binary contents of the wasm, converts it to hex and prepends a 0x
    :param path:
    :return:
    """
    logger.info("Loading runtime {}", path.__fspath__())
    with path.open("rb") as file:
        contents = file.read().hex()

    return '0x' + contents


def select_file(path : Path) -> Path:
    """
    This presents the user with a selection menu in which they can select a wasm file for upload.
    :param path:
    :return:
    """

    wasm_files = []
    for file in path.rglob("*.wasm"):
        wasm_files.append(file)

    print("*** Select .wasm file to use for runtime upgrade ***")
    for (i, file) in enumerate(wasm_files):
        print("%i) %s" % (i, file))

    try:
        selection = int(input("Selection: "))
        if 0 > selection > len(wasm_files):
            raise Exception

        file = wasm_files[selection]
        return file
    except Exception:
        logger.error("Invalid input. Aborting")
        quit(-1)



def select_runtime(file_or_dir):
    """
    This function determines if a file is given for runtime upgrade, or a directory in which the file can be found
    :param file_or_dir:
    :return:
    """
    fs_object = Path(file_or_dir)
    if fs_object.is_file():
        __validate_wasm_extension(fs_object)
        return fs_object

    if fs_object.is_dir():
        return select_file(fs_object)

    else:
        logger.error("--runtime must be a .wasm file or a directory containing one or more .wasm file in its tree. Aborting")
        quit(-1)


def __validate_wasm_extension(fs_object):
    """
    Checks if this file ends with .wasm. Aborts if not
    :param fs_object:
    :return:
    """
    if not fs_object.suffix == '.wasm':
        logger.error("Provided runtime {} is not a .wasm file. Aborting", fs_object.__fspath__())
        quit(-1)


def confirm_runtime(file: Path):
    """
    Presents the user with a final confirmation of the .wasm file to be used.  If the input is anything but 'y',
    the script aborts.
    :param file:
    :return:
    """
    confirmation = input("Are you sure you want to upgrade the network with %s ? (y/n)" % file)
    if confirmation != 'y':
        logger.error("Upgrade not confirmed. Aborting")
        quit(-1)



def do_upgrade(secret_phrase, url, payload):
    """
    Does the actual runtime upgrade.
    1) The keypair to be used is determined
    2) The substrate interface is initialize
    3) The call is prepared
    4) The extrinsic is created and signed with the keypair
    5) The extrnisic is send to the node

    :param secret_phrase:
    :param url:
    :param payload:
    :return:
    """

    logger.info("Doing runtime upgrade")

    keypair = __determine_keypair(secret_phrase)
    substrate = __init_interface(url)
    call = __prepare_upgrade_call(payload, substrate)

    extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)

    try:
        result = substrate.submit_extrinsic(extrinsic)
        logger.info("Result: {}", result)
    except Exception as e:
        logger.error(e)
    finally:
        logger.info("Done")


def __determine_keypair(secret_phrase):
    """
    Determines the keypair to use. Valid inputs are a multi word mnemonic or an URI starting with //
    :param secret_phrase:
    :return:
    """

    if secret_phrase[0:2] == "//":
        keypair = Keypair.create_from_uri(secret_phrase)
    else:
        keypair = Keypair.create_from_mnemonic(secret_phrase)
    return keypair


def __prepare_upgrade_call(payload, substrate):
    """
    Prepares the upgrade call to the blockchain, the output needs to be fed to a function that
    creates and signs an extrinsic

    :param payload:
    :param substrate:
    :return:
    """

    try:
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
        return call
    except Exception as e:
        logger.error("An error occured while composing a call to the chain. [{}]. Check the --url argument. Aborting", e)
        quit(-1)


def __init_interface(url):
    """
    Initializes the interface to the node running the chain.
    The url parameter should be a properly formatted http://<ip_or_host[:port] string

    :param url:
    :return:
    """

    custom_type_registry = {
        "runtime_id": 2,
        "types": {
            "NeuronMetadata": {
                "type": "struct",
                "type_mapping": [["ip", "u128"], ["port", "u16"], ["ip_type", "u8"]]
            }
        }
    }

    substrate = SubstrateInterface(
        url=url,
        address_type=42,
        type_registry_preset='substrate-node-template',
        type_registry=custom_type_registry
    )
    return substrate


def __validate_url(url):
    """
    Checks if the supplied url is valid
    :param url:
    :return:
    """
    if not validators.url(url):
        logger.error("{} is not a valid URL. Aborting", url)
        quit(-1)


if __name__ == '__main__':
    parser = ArgumentParser(description="Perform a runtime upgrade of a substrate blockchain")
    parser.add_argument("--url", required=True, dest="url", help="The url of the node's RPC server")
    parser.add_argument("--runtime", required=True, dest="runtime",
                        help="The path to the runtime that needs to be uploaded. "
                        "This can either be to a .wasm file, or a directory containing the file in its tree.")

    args = parser.parse_args()
    url = args.url
    runtime = args.runtime

    __validate_url(url)

    wasm_file = select_runtime(runtime)
    confirm_runtime(wasm_file)
    payload = read_file(wasm_file)

    secret_phrase = input("Secret phrase for sudo key: ")

    do_upgrade(secret_phrase, url, payload)
    secret_phrase = "X" * 500 # Erases the secret phrase from memory (hopefully)




