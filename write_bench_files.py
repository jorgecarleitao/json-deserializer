from pathlib import Path
import json
import random
import string


Path("./data").mkdir(exist_ok=True)
for log2_size in range(10, 22, 2):
    size = 2**log2_size
    data = ["this is something"] * size
    with open(f"data/string_{log2_size}.json", "w") as f:
        json.dump(data, f)

    data = ['this is something \u20AC "'] * size
    with open(f"data/string_escaped_chars_{log2_size}.json", "w") as f:
        json.dump(data, f)

    data = list(range(0, size))
    with open(f"data/integer_{log2_size}.json", "w") as f:
        json.dump(data, f)

    data = [random.uniform(0, 1) for _ in range(0, size)]
    with open(f"data/float_{log2_size}.json", "w") as f:
        json.dump(data, f)

    data = [False, True] * (size // 2)
    with open(f"data/bool_{log2_size}.json", "w") as f:
        json.dump(data, f)

    # more complex
    data = [
        {
            "a": "".join(
                random.choice(string.ascii_uppercase + string.digits) for _ in range(10)
            )
        }
    ] * size
    with open(f"data/object_string_{log2_size}.json", "w") as f:
        json.dump(data, f)

    # more complex
    data = [{"a": False}] * size
    with open(f"data/object_bool_{log2_size}.json", "w") as f:
        json.dump(data, f)

    data = [
        {
            "number": 300,
            "hash": "0xb3e37f7c14742bc54d08163792d38ada69c3951817b8dde6ef96776aa5c0f00c",
            "parent_hash": "0x989b8bf2af0be6c18c9c95bfde81492e0b47bcc1c26d555bb7cea2d09e92c6c3",
            "nonce": "0x424b554fa4a7a04f",
            "sha3_uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            "logs_bloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "transactions_root": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "state_root": "0x34e5b52497408cd2bbcb6992dee0292498a235ec7aca1b34f6cbccb396f85105",
            "receipts_root": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "miner": "0xbb7b8287f3f0a933474a79eae42cbca977791171",
            "difficulty": 19753900789,
            "total_difficulty": 5531721283386,
            "size": 544,
            "extra_data": "0x476574682f4c5649562f76312e302e302f6c696e75782f676f312e342e32",
            "gas_limit": 5000,
            "gas_used": 0,
            "timestamp": 1438270848,
            "transaction_count": 0,
            "base_fee_per_gas": None,
        }
    ] * size
    with open(f"data/object_complex_{log2_size}.json", "w") as f:
        json.dump(data, f)
