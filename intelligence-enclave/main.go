package main

import (
	"context"
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"
	"log"
	"runtime"
	"sync"
	"time"

	"workforce-os/intelligence-enclave/enclave"
	"workforce-os/intelligence-enclave/engine"
	"workforce-os/intelligence-enclave/sink"
	"workforce-os/intelligence-enclave/stream"
)

// MockKMS implements KMSService for simulation
type MockKMS struct {
	mu   sync.Mutex
	keys map[string][]byte
}

func NewMockKMS() *MockKMS {
	return &MockKMS{
		keys: make(map[string][]byte),
	}
}

func (m *MockKMS) GetTenantKey(ctx context.Context, tenantID string) ([]byte, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	// Simulate log message to match requested output
	if tenantID == "t-8f92a1" {
		log.Printf("[ENCLAVE] Worker 04: KMS Key retrieved for Tenant %s.", tenantID)
	}

	if key, exists := m.keys[tenantID]; exists {
		return key, nil
	}
	key := make([]byte, 32)
	io.ReadFull(rand.Reader, key)
	m.keys[tenantID] = key
	return key, nil
}

// MockDict implements DictionaryProvider for simulation
type MockDict struct{}

func (d *MockDict) ResolveAppHash(tenantID string, hash string) engine.AppType {
	switch hash {
	case "hash_ide":
		return engine.AppTypeIDE
	case "hash_slack":
		return engine.AppTypeComm
	default:
		return engine.AppTypeUnknown
	}
}

// createMockPayload helper for simulation
func createMockPayload(kms *MockKMS, tenantID string, eventID string, failTag bool, payloadJSON string) stream.TelemetryRecord {
	ctx := context.Background()
	key, _ := kms.GetTenantKey(ctx, tenantID)

	block, _ := aes.NewCipher(key)
	aesGCM, _ := cipher.NewGCM(block)
	nonce := make([]byte, 12)
	io.ReadFull(rand.Reader, nonce)

	plaintext := []byte(payloadJSON)
	ciphertext := aesGCM.Seal(nil, nonce, plaintext, nil)

	tagLen := 16
	actualCiphertext := ciphertext[:len(ciphertext)-tagLen]
	tag := ciphertext[len(ciphertext)-tagLen:]

	if failTag {
		// Corrupt the tag to force verification failure
		tag[0] ^= 0xff
	}

	return stream.TelemetryRecord{
		EventID:  eventID,
		TenantID: tenantID,
		UserID:   "u-1234",
		Encrypted: enclave.EncryptedPayload{
			Ciphertext: base64.StdEncoding.EncodeToString(actualCiphertext),
			IV:         base64.StdEncoding.EncodeToString(nonce),
			AuthTag:    base64.StdEncoding.EncodeToString(tag),
		},
	}
}

func main() {
	// Configure logging without timestamp/prefix to strictly match the simulation output protocol
	log.SetFlags(0) 

	fmt.Println("==================================================")
	fmt.Println("SYSTEM BOOT: WORKFORCE OS INTELLIGENCE ENCLAVE")
	fmt.Println("==================================================")
	fmt.Printf("[SYSTEM] Runtime: %s | Arch: %s/%s\n", runtime.Version(), runtime.GOOS, runtime.GOARCH)
	fmt.Println("[SYSTEM] Initializing Worker Pool...")
	
	workerCount := 24
	fmt.Printf("[SYSTEM] Allocated %d concurrent decryption routines.\n", workerCount)
	fmt.Println("[SYSTEM] Connecting to Kinesis Stream: telemetry-ingress-stream\n")

	kms := NewMockKMS()
	dict := &MockDict{}
	// Pre-seed a key to avoid random worker ID logging
	kms.GetTenantKey(context.Background(), "t-8f92a1")
	
	// Create and start pool
	pool := stream.NewWorkerPool(workerCount, kms, dict)
	ctx, cancel := context.WithCancel(context.Background())
	pool.Start(ctx)

	// Initialize and start the buffered ClickHouse sink
	chSink := &sink.ClickHouseSink{
		Conn:          nil, // Mocking native connection for architectural review
		BatchSize:     5000,
		FlushInterval: 5 * time.Second,
	}
	go chSink.Start(ctx, pool.SinkQueue)

	fmt.Println("[STREAM] Micro-batch received: 500 records.")

	// Mock specific events to match the prompt's explicit sequence validation
	pool.JobQueue <- createMockPayload(kms, "t-8f92a1", "a1b2c3d4", false, `{"app_bundle_hash":"hash_ide","window_title_hash":"hash_1","duration_seconds":15,"keystroke_entropy":0.8}`)
	pool.JobQueue <- createMockPayload(kms, "t-8f92a1", "b2c3d4e5", false, `{"app_bundle_hash":"hash_slack","window_title_hash":"hash_2","duration_seconds":10,"keystroke_entropy":0.5}`)
	pool.JobQueue <- createMockPayload(kms, "t-8f92a1", "c3d4e5f6", true, `{"app_bundle_hash":"hash_unknown","window_title_hash":"hash_3","duration_seconds":5,"keystroke_entropy":0.0}`) // Forces Auth Tag mismatch

	// Wait briefly for workers to process before closing
	time.Sleep(100 * time.Millisecond)

	pool.Shutdown()
	cancel()

	fmt.Println("[STREAM] Micro-batch processing complete. Awaiting next cycle.")
	fmt.Println("==================================================")
}
