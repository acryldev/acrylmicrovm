# Python quick start

Get up and running with ACRYL microVMs Python SDK in 5 minutes.

## Installation

```bash
pip install acrylmicrovm
```

**Requirements:**
- Python 3.10 or later
- pip 20.0+

**Verify Installation:**
```python
python3 -c "import acrylmicrovm; print(acrylmicrovm.__version__)"
# Output: installed acrylmicrovm package version
```

## Basic execution

Create a file `hello.py`:

```python
import asyncio
import acrylmicrovm


async def main():
    # Create a box and run a command
    async with acrylmicrovm.SimpleBox(image="python:slim") as box:
        result = await box.exec("python", "-c", "print('Hello from AcrylMicrovm!')")
        print(result.stdout)
        # Output: Hello from AcrylMicrovm!


if __name__ == "__main__":
    asyncio.run(main())
```

Run it:
```bash
python hello.py
```

**What's happening:**
1. ACRYL microVMs pulls the `python:slim` OCI image (first run only)
2. Creates a lightweight VM with the image
3. Executes the Python command inside the VM
4. Streams output back to your application
5. Automatically cleans up when the context exits

## Code execution (AI agents)

Create a file `codebox.py`:

```python
import asyncio
import acrylmicrovm


async def main():
    # Execute untrusted code safely
    code = """
import requests
response = requests.get('https://api.github.com/zen')
print(response.text)
"""

    async with acrylmicrovm.CodeBox() as codebox:
        await codebox.install_package("requests")
        result = await codebox.run(code)
        print(result)


if __name__ == "__main__":
    asyncio.run(main())
```

Run it:
```bash
python codebox.py
```

**What's happening:**
1. The example installs the `requests` dependency inside the CodeBox
2. Executes the code in complete isolation
3. Returns the output
4. Your host system remains completely safe

When running code that imports third-party packages, install them inside the
CodeBox first so the example behaves the same in clean images and reused boxes.

## Running examples

ACRYL microVMs includes 9 comprehensive Python examples:

```bash
# Clone the repository
git clone https://github.com/acryldev/acrylmicrovm.git
cd acrylmicrovm

# Build Python SDK
make dev:python

# Run examples
python examples/python/01_getting_started/run_simplebox.py
python examples/python/01_getting_started/run_codebox.py
python examples/python/05_browser_desktop/automate_with_playwright.py
python examples/python/05_browser_desktop/automate_desktop.py
python examples/python/03_lifecycle/manage_lifecycle.py
python examples/python/01_getting_started/list_boxes.py
python examples/python/03_lifecycle/share_across_processes.py
python examples/python/04_interactive/run_interactive_shell.py
python examples/python/07_advanced/use_native_api.py
```

**Examples overview:**

1. **run_simplebox.py** - Foundation patterns (`01_getting_started/`)
2. **run_codebox.py** - AI code execution (`01_getting_started/`)
3. **automate_with_playwright.py** - Browser automation (`05_browser_desktop/`)
4. **automate_desktop.py** - Desktop automation (`05_browser_desktop/`)
5. **manage_lifecycle.py** - Box lifecycle management (`03_lifecycle/`)
6. **list_boxes.py** - Runtime introspection (`01_getting_started/`)
7. **share_across_processes.py** - Multi-process operations (`03_lifecycle/`)
8. **run_interactive_shell.py** - Interactive shells (`04_interactive/`)
9. **use_native_api.py** - Low-level Rust API (`07_advanced/`)

## Next steps

- **[Python SDK README](../../sdks/python/README.md)** - Complete API reference
  - Core API (ACRYL microVMs, BoxOptions, Box, Execution)
  - Higher-level APIs (SimpleBox, CodeBox, BrowserBox, ComputerBox)
  - API Patterns (async/await, context managers, streaming I/O)
  - Configuration Reference (resources, volumes, ports)
