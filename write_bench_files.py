from pathlib import Path
import json
import random
import string


Path("./data").mkdir(exist_ok=True)
for log2_size in range(10,22, 2):
    size = 2**log2_size
    data = ["this is something"] * size
    with open(f"data/string_{log2_size}.json", "w") as f:
        json.dump(data, f)

    data = ["this is something \u20AC \""] * size
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
    data = [{"a": ''.join(random.choice(string.ascii_uppercase + string.digits) for _ in range(10))}] * size
    with open(f"data/object_string_{log2_size}.json", "w") as f:
        json.dump(data, f)

    # more complex
    data = [{"a": False}] * size
    with open(f"data/object_bool_{log2_size}.json", "w") as f:
        json.dump(data, f)
