// Modified by ACRYL, 2026
// Copyright BoxLite AI (originally Daytona Platforms Inc.
// SPDX-License-Identifier: Apache-2.0

package errors

import (
	"encoding/json"
	"errors"

	apiclient "github.com/acryldev/acrylmicrovm/libs/api-client-go"
)

func ConvertOpenAPIError(err error) error {
	if err == nil {
		return nil
	}

	openapiErr := &apiclient.GenericOpenAPIError{}
	if !errors.As(err, &openapiErr) {
		return err
	}

	bodyString := string(openapiErr.Body())

	acrylmicrovmErr := &ErrorResponse{}
	if parseErr := json.Unmarshal([]byte(bodyString), acrylmicrovmErr); parseErr != nil {
		return err
	}

	return NewCustomError(acrylmicrovmErr.StatusCode, acrylmicrovmErr.Message, acrylmicrovmErr.Code)
}

func IsRetryableOpenAPIError(err error) bool {
	if err == nil {
		return false
	}

	if customErr, ok := err.(*CustomError); ok {
		return customErr.IsRetryable()
	}

	return true
}
