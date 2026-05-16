import os
import time
import subprocess
import platform

# The exact string we will inject. If this survives in memory, the test fails.
CANARY_STRING = "TOP_SECRET_M&A_PROJECT_APOLLO_992"
AGENT_BINARY = "./target/release/workforce-agent"

def trigger_macos_dump(pid):
    """Uses LLDB to force a core dump on Darwin without killing the process."""
    dump_file = f"/tmp/agent_{pid}.core"
    subprocess.run(["lldb", "-p", str(pid), "-o", f"process save-core {dump_file}", "-o", "quit"], capture_output=True)
    return dump_file

def trigger_win32_dump(pid):
    """Uses Sysinternals ProcDump to capture the Win32 heap."""
    dump_file = f"agent_{pid}.dmp"
    subprocess.run(["procdump", "-ma", str(pid), dump_file], capture_output=True)
    return dump_file

def run_memory_audit():
    print("[AUDIT] Starting Zero-Knowledge Memory Audit...")
    
    # 1. Launch the compiled agent in the background
    agent_process = subprocess.Popen([AGENT_BINARY], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    time.sleep(2) # Allow hooks to initialize

    # 2. Inject the Canary (Mocking OS behavior for the test environment)
    # In CI, we pass this via a dedicated testing named pipe or local socket
    print(f"[AUDIT] Injecting Canary String into OS Hook pipeline: {CANARY_STRING}")
    subprocess.run(["curl", "-X", "POST", "-d", CANARY_STRING, "http://localhost:9091/mock-os-event"])
    
    # Allow the Rust pipeline to hash and encrypt the string
    time.sleep(1) 

    # 3. Trigger Black-Box Memory Dump
    print(f"[AUDIT] Freezing process {agent_process.pid} and dumping heap memory...")
    if platform.system() == "Darwin":
        dump_path = trigger_macos_dump(agent_process.pid)
    else:
        dump_path = trigger_win32_dump(agent_process.pid)

    # 4. Read the raw binary dump and search for the plaintext canary
    print("[AUDIT] Scanning binary core dump for plaintext signatures...")
    with open(dump_path, "rb") as f:
        heap_data = f.read()

    # Clean up the process and dump file
    agent_process.kill()
    os.remove(dump_path)

    # 5. The Infosec Assertion
    if CANARY_STRING.encode('utf-8') in heap_data:
        print("[FATAL] Memory Leak Detected! Plaintext canary found in heap dump.")
        print("[FATAL] CI Build Failed. Garbage collection or variable dropping failed.")
        exit(1)
    else:
        print("[SUCCESS] Zero-Knowledge Verified. Canary string successfully destroyed in volatile memory.")
        exit(0)

if __name__ == "__main__":
    run_memory_audit()
