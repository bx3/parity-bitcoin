#!/usr/bin/env python
import requests
import json
import socket
import sys


def send_json(url, payload):
    headers = {"content-type": "application/json; charset=UTF-8"}
    response = requests.post(url, headers=headers, data=json.dumps(payload), verify=False)
    response_native = json.loads(response.text)
    #print(response_native)
    return response_native

def get_result_or_exit(response):
    error = response.get('error')
    if error is not None:
        print("rpc return error", error)
        error_message = error.get('message')
        return error_message

    result = response.get('result')
    return result

def gen_keypair(url):
    payload = {"jsonrpc": "2.0", "method": "generatekeypair", "params": [], "id":1 }
    return get_result_or_exit(send_json(url, payload))

# return coinbase txid
def mine_block(url, addresshash, num_block):
    payload = {"jsonrpc": "2.0", "method": "generateblocks", "params": [addresshash, num_block], "id":1 }
    return get_result_or_exit(send_json(url, payload))

def wallet_add_tx(url, txid, out_index):
    payload = {"jsonrpc": "2.0", "method": "walletaddtx", "params": [txid, out_index], "id":1 }
    get_result_or_exit(send_json(url, payload))

def update_wallet(url):
    payload = {"jsonrpc": "2.0", "method": "updatewallet", "params": [], "id":1 }
    get_result_or_exit(send_json(url, payload))

def print_blocks(url):
    payload = {"jsonrpc": "2.0", "method": "print_blocks", "params": [], "id":1 }
    get_result_or_exit(send_json(url, payload))

def get_balance(url):
    update_wallet(url)
    payload = {"jsonrpc": "2.0", "method": "getbalance", "params": [], "id":1 }
    return get_result_or_exit(send_json(url, payload))

def print_coins(url):
    update_wallet(url)
    payload = {"jsonrpc": "2.0", "method": "printcoins", "params": [], "id":1 }
    return get_result_or_exit(send_json(url, payload))

def get_addresshash(url):
    payload = {"jsonrpc": "2.0", "method": "getaddresshash", "params": [], "id":1 }
    return  get_result_or_exit(send_json(url, payload))

def shard_pay(url, addresshash, amount):
    payload = {"jsonrpc": "2.0", "method": "shardpay", "params": [addresshash, amount], "id":1 }
    return get_result_or_exit(send_json(url, payload))

def get_my_ip():
    line = ""
    with open('/etc/hosts') as f:
        for l in f:
            line = l
    my_ip = line.split()[0]
    # some hack
    if my_ip[0] == 'f':
        my_ip = "127.0.0.1"

    return my_ip

def get_other_ips(all_ip):
    #all_ips = set(ip_list)
    # single node simulation
    print('are you kidding')
    my_ip = get_my_ip()
    if my_ip == "127.0.0.1":
        return [my_ip]

    if len(all_ip)==0:
        print("no ip available")
        return

    if my_ip in all_ip:
        all_ip.remove(my_ip)
    else:
        print("local ip not available in ip list")
        return

    if len(all_ip)==0:
        print("no ip available")
    return all_ip

if __name__ == '__main__':
    num_args = len(sys.argv)
    cmd_list = "gen_keypair mine_block wallet_add_tx update_wallet print_blocks get_balance get_addresshash shard_pay"
    if num_args<2:
        print("Arg error")
        print("    ip:port method ..., ... is method specific")
        print("    for example, 127.0.0.1:18443 get_balance")
        print(cmd_list)
        sys.exit()
    ip_port = sys.argv[1]
    url = "http://" + ip_port
    method = sys.argv[2]
    if method == "gen_keypair":
        print(gen_keypair(url))
    elif method == "mine_block":
        addresshash, num_block = sys.argv[3], sys.argv[4]
        print(mine_block(url, addresshash, num_block))
    elif method == "wallet_add_tx":
        txid, out_index = sys.argv[3], sys.argv[4]
        print(add_tx_to_wallet(url, txid, out_index))
    elif method == "update_wallet":
        print(update_wallet(url))
    elif method == "print_blocks":
        print(print_blocks(url))
    elif method == "get_balance":
        print(get_balance(url))
    elif method == "get_addresshash":
        print(get_addresshash(url))
    elif method == "shard_pay":
        addresshash, amount = sys.argv[3], sys.argv[4]
        print(shard_pay(url, addresshash, amount))
    else:
        print("Arg error")
        print("no method " + method)
        print(cmd_list)


        


    





