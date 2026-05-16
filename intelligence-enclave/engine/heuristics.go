package engine

import (
	"encoding/json"
	"errors"
)

// DecryptedPayload matches the nested JSON context and metrics schema
type DecryptedPayload struct {
	AppBundleHash   string  `json:"app_bundle_hash"`
	WindowTitleHash string  `json:"window_title_hash"`
	DurationSeconds int     `json:"duration_seconds"`
	KeystrokeEntropy float64 `json:"keystroke_entropy"`
}

// WorkCategory defines the standardized enterprise output states
type WorkCategory string

const (
	CategoryDeepWork      WorkCategory = "DEEP_WORK"
	CategoryCollaboration WorkCategory = "COLLABORATION"
	CategoryAdmin         WorkCategory = "ADMINISTRATIVE"
	CategoryUnclassified  WorkCategory = "UNCLASSIFIED"
)

// AppType defines the recognized software class
type AppType string

const (
	AppTypeIDE        AppType = "IDE"
	AppTypeDoc        AppType = "DOCUMENT_EDITOR"
	AppTypeComm       AppType = "COMMUNICATION"
	AppTypeBrowser    AppType = "BROWSER"
	AppTypeUnknown    AppType = "UNKNOWN"
)

// DictionaryProvider interfaces with the tenant software catalog cache
type DictionaryProvider interface {
	ResolveAppHash(tenantID string, hash string) AppType
}

// ClassifiedEvent represents the final struct routed to ClickHouse
type ClassifiedEvent struct {
	EventID         string
	TenantID        string
	UserID          string
	Category        WorkCategory
	DurationSeconds int
	Entropy         float64
}

// Categorize executes the deterministic logic against the volatile payload
func Categorize(payloadBytes []byte, dict DictionaryProvider, tenantID string) (WorkCategory, DecryptedPayload, error) {
	var payload DecryptedPayload
	if err := json.Unmarshal(payloadBytes, &payload); err != nil {
		return CategoryUnclassified, payload, errors.New("failed to parse decrypted payload")
	}

	appType := dict.ResolveAppHash(tenantID, payload.AppBundleHash)

	// Thresholds: In production, these should be tenant-configurable variables
	const entropyDeepWorkThreshold = 0.6
	const entropyIdleThreshold = 0.1

	// Rule 1: High focus applications with sustained cognitive input
	if (appType == AppTypeIDE || appType == AppTypeDoc) && payload.KeystrokeEntropy >= entropyDeepWorkThreshold {
		return CategoryDeepWork, payload, nil
	}

	// Rule 2: Communication and synchronous alignment
	if appType == AppTypeComm {
		return CategoryCollaboration, payload, nil
	}

	// Rule 3: Low input density or standard administrative navigation
	if appType == AppTypeBrowser || payload.KeystrokeEntropy <= entropyIdleThreshold {
		return CategoryAdmin, payload, nil
	}

	return CategoryUnclassified, payload, nil
}
