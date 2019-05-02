#!/usr/bin/env python

import requests
import json
import socket
import subprocess
import os
from time import sleep
from wallet_basic import *

def init_addr_with_fund(url):
    address_hash = gen_keypair(url)
    print("my addresshash", address_hash)
    txid = mine_block(url, address_hash, 1) # gen 1 block
    print("mined a block with coinbase txid " + txid)
    #print_blocks(url)
    wallet_add_tx(url, txid, 0) # first tx
    update_wallet(url)

def pay_peer(my_url, peer_url, amount):
    peer_addresshash = ""
    if my_url != peer_url:
        peer_addresshash = gen_keypair(peer_url)
    else:
        peer_addresshash = gen_keypair(my_url)

    print('get peer_addrhash', peer_addresshash)
    update_wallet(my_url)
    txid = shard_pay(my_url, peer_addresshash, amount)
    print('pay to peer txid', txid)
    wallet_add_tx(peer_url, txid, 0)

if __name__ == '__main__':
    num_args = len(sys.argv)
    cmd_list = "init_addr_with_fund pay_peer"
    if num_args<2:
        print("Arg error")
        print("    ip:port method ..., ... is method specific")
        print("    for example, 127.0.0.1:18443 get_balance")
        print(cmd_list)
        sys.exit()
    ip_port = sys.argv[1]
    url = "http://" + ip_port
    method = sys.argv[2]
    if method == "init_addr_with_fund":
        print(init_addr_with_fund(url))
    elif method == "pay_peer":
        peer_ip_port = sys.argv[3]   
        peer_url = "http://" + peer_ip_port
        amount = sys.argv[4]
        pay_peer(url, peer_url, amount)
        print(mine_block(url, 0, 1))
    else:
        print("Arg error")
        print("no method " + method)
        print(cmd_list)
