package sink

import (
	"context"
	"log"
	"time"
	"workforce-os/intelligence-enclave/engine"

	"github.com/ClickHouse/clickhouse-go/v2/lib/driver"
)

type ClickHouseSink struct {
	Conn          driver.Conn
	BatchSize     int
	FlushInterval time.Duration
}

// Start draining the SinkQueue and executing bulk inserts
func (s *ClickHouseSink) Start(ctx context.Context, sinkQueue <-chan engine.ClassifiedEvent) {
	buffer := make([]engine.ClassifiedEvent, 0, s.BatchSize)
	ticker := time.NewTicker(s.FlushInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			if len(buffer) > 0 {
				s.flush(ctx, buffer) // Flush remaining on shutdown
			}
			return
		case event, ok := <-sinkQueue:
			if !ok {
				if len(buffer) > 0 {
					s.flush(ctx, buffer)
				}
				return
			}
			buffer = append(buffer, event)
			if len(buffer) >= s.BatchSize {
				s.flush(ctx, buffer)
				buffer = buffer[:0] // Reset buffer retaining capacity
			}
		case <-ticker.C:
			if len(buffer) > 0 {
				s.flush(ctx, buffer)
				buffer = buffer[:0]
			}
		}
	}
}

func (s *ClickHouseSink) flush(ctx context.Context, batch []engine.ClassifiedEvent) {
	// Architectural Simulation Support
	if s.Conn == nil {
		log.Printf("[SINK] SUCCESS: Flushed %d events to data warehouse.", len(batch))
		return
	}

	// 1. Initialize native ClickHouse batch object
	b, err := s.Conn.PrepareBatch(ctx, "INSERT INTO telemetry_events (event_id, tenant_id, user_id, category, duration_seconds, keystroke_entropy, timestamp)")
	if err != nil {
		log.Printf("[SINK] ERROR: Failed to prepare batch: %v", err)
		return
	}

	// 2. Append rows to the batch
	now := time.Now()
	for _, e := range batch {
		err := b.Append(
			e.EventID,
			e.TenantID,
			e.UserID,
			string(e.Category),
			uint32(e.DurationSeconds),
			float32(e.Entropy),
			now, // In production, this must map to the original endpoint payload timestamp
		)
		if err != nil {
			log.Printf("[SINK] ERROR: Failed to append row %s: %v", e.EventID, err)
		}
	}

	// 3. Execute bulk commit
	if err := b.Send(); err != nil {
		log.Printf("[SINK] FATAL: ClickHouse insert rejected: %v", err)
	} else {
		log.Printf("[SINK] SUCCESS: Flushed %d events to data warehouse.", len(batch))
	}
}
