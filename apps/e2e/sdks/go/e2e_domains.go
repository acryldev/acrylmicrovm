// Modified by ACRYL, 2026
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strconv"

	acrylmicrovm "github.com/acryldev/acrylmicrovm/sdks/go"
)

func domainJSON(d acrylmicrovm.Domain) map[string]any {
	var target any
	if d.Target != nil {
		target = map[string]any{"type": d.Target.Type, "box": d.Target.Box, "port": d.Target.Port}
	}
	return map[string]any{"hostname": d.Hostname, "url": d.URL, "target": target}
}

func domainsMain() error {
	rt, err := acrylmicrovm.NewRest(acrylmicrovm.AcrylMicrovmRestOptions{
		URL: os.Getenv("ACRYLMICROVM_E2E_URL"), Credential: acrylmicrovm.NewApiKeyCredential(os.Getenv("ACRYLMICROVM_E2E_API_KEY")),
		PathPrefix: os.Getenv("ACRYLMICROVM_E2E_PREFIX"),
	})
	if err != nil {
		return err
	}
	defer rt.Close()
	domains, err := rt.Domains()
	if err != nil {
		return err
	}
	defer domains.Close()
	ctx := context.Background()
	var result any
	switch os.Args[1] {
	case "list":
		held, err := domains.List(ctx)
		if err != nil {
			return err
		}
		items := []map[string]any{}
		for _, d := range held.Domains {
			items = append(items, domainJSON(d))
		}
		result = map[string]any{"domains": items, "limit": held.Limit, "used": held.Used,
			"remaining": held.Remaining, "min_length": held.MinLength}
	case "unbind":
		if err := domains.Unbind(ctx, os.Args[2]); err != nil {
			return err
		}
		result = map[string]any{}
	case "bind":
		port, err := strconv.ParseUint(os.Args[3], 10, 16)
		if err != nil {
			return err
		}
		d, err := domains.Bind(ctx, os.Args[2], acrylmicrovm.DomainTarget{
			Type: acrylmicrovm.DomainTargetBox, Box: os.Getenv("ACRYLMICROVM_E2E_BOX"), Port: uint16(port),
		})
		if err != nil {
			return err
		}
		result = domainJSON(*d)
	default:
		return fmt.Errorf("unknown operation")
	}
	return json.NewEncoder(os.Stdout).Encode(result)
}

func main() {
	if err := domainsMain(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
