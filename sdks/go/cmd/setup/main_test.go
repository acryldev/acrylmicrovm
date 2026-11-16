// Modified by ACRYL, 2026
package main

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestDownloadToModCacheExtractsLibraryAndHeader(t *testing.T) {
	t.Parallel()

	archive := createTestArchive(t, map[string]string{
		"acrylmicrovm-c-v0.8.2-darwin-arm64/lib/libacrylmicrovm.a":  "test archive",
		"acrylmicrovm-c-v0.8.2-darwin-arm64/include/acrylmicrovm.h": "#ifndef ACRYLMICROVM_H\n",
	})
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(archive)
	}))
	defer server.Close()

	destDir := t.TempDir()
	if err := os.Chmod(destDir, 0o555); err != nil {
		t.Fatalf("Chmod destDir read-only: %v", err)
	}
	t.Cleanup(func() {
		_ = os.Chmod(destDir, 0o755)
	})

	if err := downloadToModCache(server.URL, destDir); err != nil {
		t.Fatalf("downloadToModCache: %v", err)
	}

	libPath := filepath.Join(destDir, "libacrylmicrovm.a")
	headerPath := filepath.Join(destDir, "include", "acrylmicrovm.h")

	libData, err := os.ReadFile(libPath)
	if err != nil {
		t.Fatalf("ReadFile libacrylmicrovm.a: %v", err)
	}
	if string(libData) != "test archive" {
		t.Fatalf("libacrylmicrovm.a mismatch: got %q", string(libData))
	}

	headerData, err := os.ReadFile(headerPath)
	if err != nil {
		t.Fatalf("ReadFile acrylmicrovm.h: %v", err)
	}
	if !strings.Contains(string(headerData), "#ifndef ACRYLMICROVM_H") {
		t.Fatalf("acrylmicrovm.h missing expected contents: %q", string(headerData))
	}
}

func TestDownloadToModCacheRequiresHeader(t *testing.T) {
	t.Parallel()

	archive := createTestArchive(t, map[string]string{
		"acrylmicrovm-c-v0.8.2-darwin-arm64/lib/libacrylmicrovm.a": "test archive",
	})
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(archive)
	}))
	defer server.Close()

	destDir := t.TempDir()
	if err := os.Chmod(destDir, 0o555); err != nil {
		t.Fatalf("Chmod destDir read-only: %v", err)
	}
	t.Cleanup(func() {
		_ = os.Chmod(destDir, 0o755)
	})

	err := downloadToModCache(server.URL, destDir)
	if err == nil {
		t.Fatal("expected downloadToModCache to fail when include/acrylmicrovm.h is missing")
	}
	if !strings.Contains(err.Error(), "include/acrylmicrovm.h not found") {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestDownloadToModCacheRequiresLibrary(t *testing.T) {
	t.Parallel()

	archive := createTestArchive(t, map[string]string{
		"acrylmicrovm-c-v0.8.2-darwin-arm64/include/acrylmicrovm.h": "#ifndef ACRYLMICROVM_H\n",
	})
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(archive)
	}))
	defer server.Close()

	destDir := t.TempDir()
	if err := os.Chmod(destDir, 0o555); err != nil {
		t.Fatalf("Chmod destDir read-only: %v", err)
	}
	t.Cleanup(func() {
		_ = os.Chmod(destDir, 0o755)
	})

	err := downloadToModCache(server.URL, destDir)
	if err == nil {
		t.Fatal("expected downloadToModCache to fail when libacrylmicrovm.a is missing")
	}
	if !strings.Contains(err.Error(), "libacrylmicrovm.a not found") {
		t.Fatalf("unexpected error: %v", err)
	}
}

func createTestArchive(t *testing.T, files map[string]string) []byte {
	t.Helper()

	var archive bytes.Buffer
	gz := gzip.NewWriter(&archive)
	tw := tar.NewWriter(gz)

	for name, content := range files {
		data := []byte(content)
		hdr := &tar.Header{
			Name: name,
			Mode: 0o644,
			Size: int64(len(data)),
		}
		if err := tw.WriteHeader(hdr); err != nil {
			t.Fatalf("WriteHeader(%s): %v", name, err)
		}
		if _, err := tw.Write(data); err != nil {
			t.Fatalf("Write(%s): %v", name, err)
		}
	}

	if err := tw.Close(); err != nil {
		t.Fatalf("tar close: %v", err)
	}
	if err := gz.Close(); err != nil {
		t.Fatalf("gzip close: %v", err)
	}

	return archive.Bytes()
}
