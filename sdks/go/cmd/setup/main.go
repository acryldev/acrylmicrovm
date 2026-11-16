// Modified by ACRYL, 2026
// Setup downloads the prebuilt libacrylmicrovm native library from GitHub Releases
// into the Go module cache so that `go build` can compile and link against it.
//
// Usage (after go get):
//
//	go run github.com/acryldev/acrylmicrovm/sdks/go/cmd/setup
//
// The tool detects your platform and SDK version automatically.
// Set GITHUB_TOKEN to avoid API rate limits.
package main

import (
	"archive/tar"
	"compress/gzip"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"runtime/debug"
	"strings"
	"time"
)

const (
	repo          = "acryldev/acrylmicrovm"
	modulePath    = "github.com/acryldev/acrylmicrovm/sdks/go"
	archivePrefix = "acrylmicrovm-c-"
)

var httpClient = &http.Client{Timeout: 5 * time.Minute}

func main() {
	platform := detectPlatform()
	version := detectVersion()
	modCache := goModCache()

	fmt.Printf("Platform:  %s\n", platform)
	fmt.Printf("Version:   %s\n", version)
	fmt.Printf("Mod cache: %s\n", modCache)

	// Target: root module cache directory
	moduleDir := filepath.Join(modCache, modulePath+"@"+version)

	targetLib := filepath.Join(moduleDir, "libacrylmicrovm.a")
	targetHeader := filepath.Join(moduleDir, "include", "acrylmicrovm.h")
	if fileExists(targetLib) && fileExists(targetHeader) {
		fmt.Printf(
			"\nAcrylMicrovm native assets already exist at %s and %s, skipping.\n",
			targetLib,
			targetHeader,
		)
		return
	}

	if _, err := os.Stat(moduleDir); os.IsNotExist(err) {
		fatalf("module cache directory not found: %s\nRun 'go get %s@%s' first.", moduleDir, modulePath, version)
	}

	// Download from GitHub Releases
	archiveName := fmt.Sprintf("%sv%s-%s.tar.gz", archivePrefix, strings.TrimPrefix(version, "v"), platform)
	url := fmt.Sprintf("https://github.com/%s/releases/download/%s/%s", repo, version, archiveName)
	fmt.Printf("\nDownloading: %s\n", url)

	if err := downloadToModCache(url, moduleDir); err != nil {
		fatalf("download failed: %v", err)
	}

	fmt.Printf("\nSetup complete. You can now run: go build ./...\n")
}

// detectPlatform maps GOOS/GOARCH to the ACRYL microVMs platform target name.
func detectPlatform() string {
	switch {
	case runtime.GOOS == "darwin" && runtime.GOARCH == "arm64":
		return "darwin-arm64"
	case runtime.GOOS == "linux" && runtime.GOARCH == "amd64":
		return "linux-x64-gnu"
	default:
		fatalf("unsupported platform: %s/%s", runtime.GOOS, runtime.GOARCH)
		return ""
	}
}

// detectVersion finds the SDK version from build info or go.mod.
func detectVersion() string {
	// Try debug.ReadBuildInfo — works when run via `go run`
	if bi, ok := debug.ReadBuildInfo(); ok {
		// Check if this binary's main module is the SDK itself
		if bi.Main.Path == modulePath && bi.Main.Version != "(devel)" && bi.Main.Version != "" {
			return bi.Main.Version
		}
		// Check dependencies (when run from user's project)
		for _, dep := range bi.Deps {
			if dep.Path == modulePath {
				return dep.Version
			}
		}
	}

	// Fallback: parse go.mod in current directory
	data, err := os.ReadFile("go.mod")
	if err == nil {
		for _, line := range strings.Split(string(data), "\n") {
			line = strings.TrimSpace(line)
			if strings.HasPrefix(line, modulePath+" ") {
				parts := strings.Fields(line)
				if len(parts) >= 2 {
					return parts[1]
				}
			}
		}
	}

	fatalf("cannot detect SDK version: run this tool from a project that imports %s, or pin a version explicitly:\n  go run %s/cmd/setup@latest", modulePath, modulePath)
	return ""
}

// goModCache returns the Go module cache directory.
func goModCache() string {
	// Check env first (fast path)
	if cache := os.Getenv("GOMODCACHE"); cache != "" {
		return cache
	}
	// Shell out to `go env`
	out, err := exec.Command("go", "env", "GOMODCACHE").Output()
	if err != nil {
		fatalf("cannot determine GOMODCACHE: %v", err)
	}
	cache := strings.TrimSpace(string(out))
	if cache == "" {
		fatalf("GOMODCACHE is empty")
	}
	return cache
}

// downloadToModCache downloads the archive and extracts the Go SDK native assets
// into the module cache directory, temporarily making it writable.
func downloadToModCache(url, destDir string) error {
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return err
	}
	if token := os.Getenv("GITHUB_TOKEN"); token != "" {
		req.Header.Set("Authorization", "token "+token)
	}

	resp, err := httpClient.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("HTTP %d (check version and platform)", resp.StatusCode)
	}

	gz, err := gzip.NewReader(resp.Body)
	if err != nil {
		return fmt.Errorf("gzip: %w", err)
	}
	defer gz.Close()

	// Make the module cache directory writable temporarily
	if err := os.Chmod(destDir, 0o755); err != nil {
		return fmt.Errorf("chmod writable %s: %w (try running with appropriate permissions)", destDir, err)
	}
	defer func() { _ = os.Chmod(destDir, 0o555) }()

	tr := tar.NewReader(gz)
	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("tar: %w", err)
		}
		if hdr.Typeflag != tar.TypeReg {
			continue
		}

		// Archive structure: acrylmicrovm-c-vX.Y.Z-platform/{lib,include}/filename
		parts := strings.SplitN(hdr.Name, "/", 2)
		if len(parts) < 2 {
			continue
		}
		switch parts[1] {
		case "lib/libacrylmicrovm.a":
			dest := filepath.Join(destDir, "libacrylmicrovm.a")
			if err := writeFile(dest, tr, hdr.Mode); err != nil {
				return err
			}
			fmt.Printf("  extracted: libacrylmicrovm.a (%d MB)\n", hdr.Size/(1024*1024))
		case "include/acrylmicrovm.h":
			dest := filepath.Join(destDir, "include", "acrylmicrovm.h")
			if err := writeFile(dest, tr, hdr.Mode); err != nil {
				return err
			}
			fmt.Printf("  extracted: include/acrylmicrovm.h\n")
		}
	}

	if !fileExists(filepath.Join(destDir, "libacrylmicrovm.a")) {
		return fmt.Errorf("libacrylmicrovm.a not found in archive")
	}
	if !fileExists(filepath.Join(destDir, "include", "acrylmicrovm.h")) {
		return fmt.Errorf("include/acrylmicrovm.h not found in archive")
	}
	return nil
}

func writeFile(path string, r io.Reader, mode int64) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return fmt.Errorf("mkdir %s: %w", filepath.Dir(path), err)
	}
	f, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, os.FileMode(mode)&0o755)
	if err != nil {
		return fmt.Errorf("create %s: %w", path, err)
	}
	if _, err := io.Copy(f, r); err != nil {
		f.Close()
		return fmt.Errorf("write %s: %w", path, err)
	}
	return f.Close()
}

func fileExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

func fatalf(format string, args ...any) {
	fmt.Fprintf(os.Stderr, "acrylmicrovm-setup: "+format+"\n", args...)
	os.Exit(1)
}
