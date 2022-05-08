from pathlib import Path
import json


Path("./data").mkdir(exist_ok=True)
for log2_size in range(10,22, 2):
    size = 2**log2_size
    data = ["this is something"] * size
    with open(f"data/string_{log2_size}.json", "w") as f:
        json.dump(data, f)
    
    data = list(range(0, size))
    with open(f"data/number_{log2_size}.json", "w") as f:
        json.dump(data, f)

    data = [False, True] * (size // 2)
    with open(f"data/bool_{log2_size}.json", "w") as f:
        json.dump(data, f)
