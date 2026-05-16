package enclave

import (
	"context"
	"crypto/aes"
	"crypto/cipher"
	"encoding/base64"
	"errors"
	"fmt"
)

// EncryptedPayload represents the nested structure from Schema v1.1
type EncryptedPayload struct {
	Ciphertext string `json:"ciphertext"`
	IV         string `json:"iv"`
	AuthTag    string `json:"auth_tag"`
}

// KMSService defines the interface for tenant key retrieval
type KMSService interface {
	GetTenantKey(ctx context.Context, tenantID string) ([]byte, error)
}

// DecryptInVolatileMemory performs AES-256-GCM decryption.
// CRITICAL: The returned byte slice must not be logged or persisted.
func DecryptInVolatileMemory(ctx context.Context, kms KMSService, tenantID string, payload EncryptedPayload) ([]byte, error) {
	// 1. Retrieve the tenant-specific symmetric key via AWS KMS
	key, err := kms.GetTenantKey(ctx, tenantID)
	if err != nil {
		return nil, fmt.Errorf("KMS access denied for tenant %s: %v", tenantID, err)
	}

	// 2. Decode Base64 cryptographic components
	ciphertext, err := base64.StdEncoding.DecodeString(payload.Ciphertext)
	if err != nil {
		return nil, errors.New("invalid ciphertext encoding")
	}

	iv, err := base64.StdEncoding.DecodeString(payload.IV)
	if err != nil {
		return nil, errors.New("invalid IV encoding")
	}

	authTag, err := base64.StdEncoding.DecodeString(payload.AuthTag)
	if err != nil {
		return nil, errors.New("invalid auth tag encoding")
	}

	// Append the auth tag to the ciphertext for standard Go AES-GCM verification
	ciphertextWithTag := append(ciphertext, authTag...)

	// 3. Execute AES-256-GCM Decryption
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}

	aesGCM, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}

	plaintext, err := aesGCM.Open(nil, iv, ciphertextWithTag, nil)
	if err != nil {
		return nil, errors.New("authentication tag verification failed or payload altered")
	}

	return plaintext, nil
}
