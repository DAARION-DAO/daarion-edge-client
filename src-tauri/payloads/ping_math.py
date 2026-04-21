import sys
import json

def main():
    try:
        if len(sys.argv) < 2:
            raise ValueError("Missing payload arguments")
        
        raw_input = sys.argv[1]
        data = json.loads(raw_input)
        value = data.get("value")
        
        if not isinstance(value, (int, float)):
             raise ValueError("value must be numeric")
             
        # "Sandbox" logic
        result = value * 2
        
        # Bounded deterministic output via STDOUT
        print(json.dumps({"output": result}))
        sys.exit(0)
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)

if __name__ == "__main__":
    main()
