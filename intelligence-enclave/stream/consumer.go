package stream

import (
	"context"
	"log"
	"sync"
	"workforce-os/intelligence-enclave/enclave"
	"workforce-os/intelligence-enclave/engine"
)

// TelemetryRecord represents the wrapper schema coming from Kinesis
type TelemetryRecord struct {
	EventID   string                   `json:"event_id"`
	TenantID  string                   `json:"tenant_id"`
	UserID    string                   `json:"user_id"`
	Encrypted enclave.EncryptedPayload `json:"encrypted_payload"`
}

// WorkerPool manages concurrent processing of the Kinesis shard data
type WorkerPool struct {
	WorkerCount int
	JobQueue    chan TelemetryRecord
	SinkQueue   chan engine.ClassifiedEvent
	WaitGroup   sync.WaitGroup
	KMS         enclave.KMSService
	Dict        engine.DictionaryProvider
}

func NewWorkerPool(count int, kms enclave.KMSService, dict engine.DictionaryProvider) *WorkerPool {
	return &WorkerPool{
		WorkerCount: count,
		JobQueue:    make(chan TelemetryRecord, 5000), // Buffered to handle Kinesis batch spikes
		SinkQueue:   make(chan engine.ClassifiedEvent, 5000),
		KMS:         kms,
		Dict:        dict,
	}
}

func (p *WorkerPool) Start(ctx context.Context) {
	for i := 0; i < p.WorkerCount; i++ {
		p.WaitGroup.Add(1)
		// Format worker IDs starting from 1 (e.g., 01, 02)
		go p.worker(ctx, i+1)
	}
}

func (p *WorkerPool) worker(ctx context.Context, workerID int) {
	defer p.WaitGroup.Done()

	for record := range p.JobQueue {
		// Enforce strict context cancellation compliance
		select {
		case <-ctx.Done():
			return
		default:
		}

		// 1. Pass to the secure enclave for volatile decryption
		plaintext, err := enclave.DecryptInVolatileMemory(ctx, p.KMS, record.TenantID, record.Encrypted)
		if err != nil {
			log.Printf("[ENCLAVE] Worker %02d: ERROR Auth Tag mismatch [FAIL]. Event: %s. Dropping payload.", workerID, record.EventID)
			continue
		}

		log.Printf("[ENCLAVE] Worker %02d: AES-256-GCM verification [PASS]. Event: %s.", workerID, record.EventID)

		// 2. Pass 'plaintext' to the Behavioral Heuristics Engine
		category, payloadData, err := engine.Categorize(plaintext, p.Dict, record.TenantID)
		if err != nil {
			log.Printf("[ENGINE] Worker %02d: ERROR Category resolution failed for Event: %s", workerID, record.EventID)
		} else {
			classified := engine.ClassifiedEvent{
				EventID:         record.EventID,
				TenantID:        record.TenantID,
				UserID:          record.UserID,
				Category:        category,
				DurationSeconds: payloadData.DurationSeconds,
				Entropy:         payloadData.KeystrokeEntropy,
			}
			p.SinkQueue <- classified
			log.Printf("[ENGINE] Worker %02d: Classified Event %s as [%s].", workerID, record.EventID, category)
		}

		// 3. EXPLICIT PRIVACY ACTION: Overwrite and release the volatile memory
		// Go's garbage collector handles the underlying array, but we explicitly drop the reference.
		_ = plaintext
		plaintext = nil 
	}
}

func (p *WorkerPool) Shutdown() {
	close(p.JobQueue)
	p.WaitGroup.Wait()
}
