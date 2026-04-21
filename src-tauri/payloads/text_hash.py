import sys
import json
import hashlib

def main():
    try:
        raw_in = sys.argv[1]
        data = json.loads(raw_in)
        text = data.get("text", "")
        
        # Deterministic text hashing (SHA-256)
        out = hashlib.sha256(text.encode('utf-8')).hexdigest()
        print(json.dumps({"output": out, "error": None}))
    except Exception as e:
        print(json.dumps({"output": None, "error": str(e)}))

if __name__ == "__main__":
    main()
