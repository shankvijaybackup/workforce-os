import os
import base64
import json
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.exceptions import InvalidTag

def generate_key() -> bytes:
    """Generate a random 256-bit (32 byte) symmetric key."""
    return AESGCM.generate_key(bit_length=256)

def encrypt_payload(key: bytes, plaintext_dict: dict, associated_data: bytes = None) -> dict:
    """
    Encrypt a payload using AES-256-GCM.
    Returns a dictionary with base64 encoded ciphertext, iv, and auth_tag.
    """
    aesgcm = AESGCM(key)
    
    # 96-bit Initialization Vector (IV)
    iv = os.urandom(12)
    
    plaintext = json.dumps(plaintext_dict).encode('utf-8')
    
    # AESGCM.encrypt appends the 128-bit authentication tag to the ciphertext
    encrypted_data = aesgcm.encrypt(iv, plaintext, associated_data)
    
    # The tag is the last 16 bytes
    ciphertext = encrypted_data[:-16]
    auth_tag = encrypted_data[-16:]
    
    return {
        "ciphertext": base64.b64encode(ciphertext).decode('utf-8'),
        "iv": base64.b64encode(iv).decode('utf-8'),
        "auth_tag": base64.b64encode(auth_tag).decode('utf-8')
    }

def decrypt_payload(key: bytes, encrypted_payload: dict, associated_data: bytes = None) -> dict:
    """
    Decrypt a payload using AES-256-GCM.
    Fails with InvalidTag if the ciphertext or tag has been modified.
    """
    aesgcm = AESGCM(key)
    
    iv = base64.b64decode(encrypted_payload["iv"])
    ciphertext = base64.b64decode(encrypted_payload["ciphertext"])
    auth_tag = base64.b64decode(encrypted_payload["auth_tag"])
    
    # Reconstruct the expected payload for the cryptography library
    encrypted_data = ciphertext + auth_tag
    
    try:
        decrypted_data = aesgcm.decrypt(iv, encrypted_data, associated_data)
        return json.loads(decrypted_data.decode('utf-8'))
    except InvalidTag:
        raise ValueError("Decryption failed: Ciphertext or Authentication Tag was modified!")

def run_demonstration():
    print("--- Cryptographic Reference Implementation (AES-256-GCM) ---")
    
    # 1. Generate a mock 256-bit tenant key
    tenant_key = generate_key()
    print(f"[+] Generated 256-bit Tenant Key: {base64.b64encode(tenant_key).decode('utf-8')}")
    
    # 2. Prepare raw payload (what the agent captures)
    raw_payload = {
        "active_window": "VS Code - main.tf",
        "keystroke_entropy": 4.87,
        "deep_work_status": True
    }
    print(f"\n[+] Raw Payload: {raw_payload}")
    
    # Optional Associated Data (e.g., tenant_id to prevent confused deputy attacks)
    aad = b"tenant_t-8f92a1"
    
    # 3. Encrypt the payload
    encrypted = encrypt_payload(tenant_key, raw_payload, associated_data=aad)
    print("\n[+] Encrypted Payload Block:")
    print(json.dumps(encrypted, indent=2))
    
    # 4. Decrypt the payload
    print("\n[+] Attempting Valid Decryption...")
    decrypted = decrypt_payload(tenant_key, encrypted, associated_data=aad)
    print(f"[+] Decrypted Payload: {decrypted}")
    assert raw_payload == decrypted, "Decrypted data does not match original!"
    print("[+] Valid decryption successful.")
    
    # 5. Demonstrate deterministic failure on tampering
    print("\n[+] Attempting Decryption with Tampered Ciphertext...")
    tampered_encrypted = dict(encrypted)
    
    # Flip a bit in the ciphertext
    raw_ciphertext = bytearray(base64.b64decode(tampered_encrypted["ciphertext"]))
    raw_ciphertext[0] ^= 0x01
    tampered_encrypted["ciphertext"] = base64.b64encode(raw_ciphertext).decode('utf-8')
    
    try:
        decrypt_payload(tenant_key, tampered_encrypted, associated_data=aad)
        print("[-] FAILURE: Tampered data was decrypted successfully (THIS SHOULD NOT HAPPEN).")
    except ValueError as e:
        print(f"[+] SUCCESS: Tampered data rejected. Error: {e}")

if __name__ == "__main__":
    run_demonstration()
